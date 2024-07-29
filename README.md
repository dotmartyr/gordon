# Gordon

Gordon is a Discord bot built using Rust, integrating with the OpenAI API to provide responses. The bot listens to messages mentioning it and responds using OpenAI's GPT-4 model. It also has a simple `!ping` command to verify the bot is operational.

## Features

- Responds to messages that mention the bot using OpenAI's GPT-4 model.
- Supports a simple `!ping` command to check the bot's status.

## Prerequisites

- Rust and Cargo installed on your machine.
- A Discord bot token.
- An OpenAI API key.
- Shuttle.rs account for deployment.

## Setup

### Configuration

Create a `Secrets.toml` file in the root of your project with the following content:

```toml
DISCORD_TOKEN = "YOUR_DISCORD_TOKEN"
BOT_USER_ID = "YOUR_BOT_USER_ID"
OPENAI_API_KEY = "YOUR_OPENAI_API_KEY"
```

### Build
```
cargo run
```

### Usage

```
cargo install cargo-shuttle
cargo shuttle login
cargo shuttle run # run locally and test in discord.
cargo shuttle deploy
```