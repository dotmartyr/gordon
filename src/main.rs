mod handler;
mod openai;

use std::collections::HashMap;

use handler::{string_to_user_id, Handler};
use openai::OpenAIClient;
use serenity::prelude::*;
use shuttle_runtime::{Error as ShuttleError, SecretStore};

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

fn get_secrets(secrets: &SecretStore) -> Result<HashMap<String, String>, ShuttleError> {
    let required_keys = ["DISCORD_TOKEN", "DISCORD_CLIENT_ID", "OPENAI_API_KEY"];
    let mut secrets_map = HashMap::new();

    for &key in &required_keys {
        let value = secrets.get(key).ok_or_else(|| {
            ShuttleError::Custom(anyhow::anyhow!("Missing secret: {}", key).into())
        })?;
        secrets_map.insert(key.to_string(), value);
    }

    Ok(secrets_map)
}

#[shuttle_runtime::main]
async fn shuttle_main(
    #[shuttle_runtime::Secrets] secrets: shuttle_runtime::SecretStore,
) -> Result<Gordon, ShuttleError> {
    let secrets_map = get_secrets(&secrets)?;

    let token = secrets_map.get("DISCORD_TOKEN").unwrap();
    let discord_client_id = secrets_map.get("DISCORD_CLIENT_ID").unwrap();
    let openai_api_key = secrets_map.get("OPENAI_API_KEY").unwrap();

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let serenity_client = Client::builder(&token, intents)
        .event_handler(Handler {
            bot_id: string_to_user_id(&discord_client_id)?,
            openai_client: OpenAIClient::new(openai_api_key.clone()),
        })
        .await
        .expect("Err creating client");

    Ok(Gordon {
        client: serenity_client,
    })
}
