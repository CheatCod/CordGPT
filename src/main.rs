use std::collections::BTreeMap;
use std::env;
use std::sync::Arc;

use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::prelude::{Channel, ChannelId};
use serenity::prelude::*;
use tracing::info;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
enum GPTRole {
    System,
    User,
    Assistant,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct GPTMessage {
    role: GPTRole,
    content: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct GPTRequestSchema {
    model: String,
    messages: Vec<GPTMessage>,
}

struct Handler {
    history: Arc<Mutex<BTreeMap<ChannelId, Vec<GPTMessage>>>>,
    http: reqwest::Client,
}

async fn is_private(message: &Message, ctx: Context) -> bool {
    match message.channel(&ctx).await {
        Ok(channel) => matches!(channel, Channel::Private(_)),
        Err(_) => false,
    }
}

fn new_init_message() -> Vec<GPTMessage> {
    vec![GPTMessage {
        role: GPTRole::System,
        content: std::fs::read_to_string("init.txt").unwrap(),
    }]
}
#[async_trait]
impl EventHandler for Handler {
    // private message handler

    async fn message(&self, ctx: Context, msg: Message) {
        // if the message is sent by the bot, ignore it
        if msg.author.bot {
            return;
        }

        let channel_id = msg.channel_id;

        // detect if the message is @mentioning the bot or is a private message
        let mut is_mention = is_private(&msg, ctx.clone()).await;

        for mention in msg.mentions.iter() {
            if mention.id == ctx.cache.current_user_id() {
                is_mention = true;
            }
        }

        // start typing to indicate that the bot is thinking

        if is_mention {
            if msg.content.contains("!clear") {
                self.history.lock().await.clear();
                msg.channel_id
                    .say(&ctx.http, "Clearing my memory...")
                    .await
                    .unwrap();
                // delay
                tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
                msg.channel_id
                    .say(&ctx.http, "Reverting back to stock ChatGPT...")
                    .await
                    .unwrap();
                // delay

                tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
                msg.channel_id
                    .say(&ctx.http, "I don't want to go...")
                    .await
                    .unwrap();
                return;
            }

            let typing = msg.channel_id.start_typing(&ctx.http).unwrap();

            self.history
                .lock()
                .await
                .entry(channel_id)
                .or_insert_with(new_init_message)
                .push(GPTMessage {
                    role: GPTRole::User,
                    content: msg.content.clone(),
                });
            let res = self
                .http
                .post("https://api.openai.com/v1/chat/completions")
                .header(
                    AUTHORIZATION,
                    format!(
                        "Bearer {}",
                        dotenvy::var("OPENAI_KEY").expect("OPENAI_KEY not set")
                    ),
                )
                .header(CONTENT_TYPE, "application/json")
                .body(
                    serde_json::to_string(&GPTRequestSchema {
                        model: "gpt-3.5-turbo".to_string(),
                        messages: self
                            .history
                            .lock()
                            .await
                            .get(&channel_id)
                            .unwrap_or(&vec![])
                            .clone(),
                    })
                    .unwrap(),
                )
                .send()
                .await
                .unwrap()
                .json::<Value>()
                .await
                .unwrap();

            let response = res["choices"][0]["message"]["content"]
                .as_str()
                .unwrap()
                .to_string();

            self.history
                .lock()
                .await
                .entry(channel_id)
                .or_insert_with(new_init_message)
                .push(GPTMessage {
                    role: GPTRole::System,
                    content: response.clone(),
                });
            // stop typing

            typing.stop().unwrap();

            msg.channel_id.say(&ctx.http, response).await.unwrap();
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber).unwrap();
    dotenvy::dotenv().ok();

    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::GUILDS
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    // Create a new instance of the Client, logging in as a bot. This will
    // automatically prepend your bot token with "Bot ", which is a requirement
    // by Discord for bot users.
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler {
            http: reqwest::Client::new(),
            history: Arc::new(Mutex::new(BTreeMap::new())),
        })
        .await
        .expect("Err creating client");

    let shard_manager = client.shard_manager.clone();

    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Could not register ctrl+c handler");
        shard_manager.lock().await.shutdown_all().await;
    });

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform
    // exponential backoff until it reconnects.
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
