# CordGPT

Power of ChatGPT in your Discord server or DM with a simple mention.

# Why?

Created to help the developers of [Lodestone](https://github.com/Lodestone-Team)
to aid in their development of the project.

It is much faster than the free version of ChatGPT, and it costs way less than
20$ a month.

Also texting a bot in discord is much easier than using a website IMO.

# Setup

1. Install the Rust toolchain from
   [here](https://www.rust-lang.org/tools/install)
2. Clone the repo
3. Run `cargo build --release`
4. While that is building, create a Discord bot and an OpenAI account. First
   google search gave me this
   (https://www.ionos.com/digitalguide/server/know-how/creating-discord-bot/),
   make sure to enable `MESSAGE CONTENT INTENT` in the bot settings on the discord developer portal.
5. Fill out the .env file with your Discord ot and OpenAI token
6. Fill in init.txt with the text you want to start the bot with
7. Run `cargo run --release` or `./target/release/cordgpt`
