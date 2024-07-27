use anyhow::Result;
use dotenv::dotenv;
use serenity::async_trait;
use serenity::client::bridge::gateway::GatewayIntents;
use serenity::model::gateway::Ready;
use serenity::model::prelude::Message;
use serenity::prelude::*;
use shuttle_runtime::Error as ShuttleError;
use std::env;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!ping" {
            if let Err(why) = msg.channel_id.say(&ctx.http, "pong").await {
                println!("Error sending message: {:?}", why);
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}
struct Gordon {
    client: Client,
}

#[shuttle_runtime::async_trait]
impl shuttle_runtime::Service for Gordon {
    async fn bind(mut self, _addr: std::net::SocketAddr) -> Result<(), ShuttleError> {
        self.client
            .start()
            .await
            .map_err(|e| ShuttleError::Custom(e.into()))
    }
}
#[shuttle_runtime::main]
async fn shuttle_main() -> Result<Gordon, ShuttleError> {
    dotenv().ok();
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::empty();
    let client = Client::builder(&token)
        .event_handler(Handler)
        .intents(intents)
        .await
        .map_err(|e| ShuttleError::Custom(e.into()))?;

    Ok(Gordon { client })
}
