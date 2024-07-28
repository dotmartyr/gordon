use anyhow::Result;

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::prelude::*;

use shuttle_runtime::Error as ShuttleError;

struct Handler;

struct Gordon {
    client: Client,
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!ping" {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                println!("Error sending message: {why:?}");
            }
        }
    }
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
async fn shuttle_main(
    #[shuttle_runtime::Secrets] secrets: shuttle_runtime::SecretStore,
) -> Result<Gordon, ShuttleError> {
    let token = secrets
        .get("DISCORD_TOKEN")
        .expect("Expected a token in the environment");

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }

    Ok(Gordon { client })
}
