use anyhow::{anyhow, Result};
use reqwest::Client as ReqwestClient;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::prelude::UserId;
use serenity::prelude::*;
use shuttle_runtime::Error as ShuttleError;

struct Handler {
    bot_id: UserId,
    openai_client: ReqwestClient,
    openai_api_key: String,
}

struct Gordon {
    client: Client,
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.mentions_user_id(self.bot_id) {
            let response = self.get_openai_response(&msg.content).await;

            match response {
                Ok(reply) => {
                    if let Err(why) = msg.channel_id.say(&ctx.http, reply).await {
                        println!("Error sending message: {:?}", why);
                    }
                }
                Err(err) => {
                    println!("Error getting OpenAI response: {:?}", err);
                }
            }
        } else if msg.content == "!ping" {
            if let Err(why) = msg.channel_id.say(&ctx.http, "pong").await {
                println!("Error sending message: {:?}", why);
            }
        }
    }

    async fn ready(&self, _ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

impl Handler {
    async fn get_openai_response(&self, user_message: &str) -> Result<String> {
        let prompt = format!(
            "{} Also, please answer with a nerdy flair but try not to be overly verbose.",
            user_message
        );

        let request_body = serde_json::json!({
            "model": "gpt-4o",
            "messages": [{"role": "user", "content": prompt}],
            "max_tokens": 4096,
            "temperature": 0.7,
            "top_p": 1.0,
            "frequency_penalty": 0.0,
            "presence_penalty": 0.0
        });

        let response = self
            .openai_client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.openai_api_key))
            .json(&request_body)
            .send()
            .await?;

        if response.status().is_success() {
            let response_body: serde_json::Value = response.json().await?;
            let reply = response_body["choices"][0]["message"]["content"]
                .as_str()
                .unwrap_or("I'm sorry, I couldn't understand that.")
                .to_string();
            Ok(reply)
        } else {
            Err(anyhow!(
                "Failed to get response from OpenAI API: {}",
                response.status()
            ))
        }
    }
}

fn string_to_user_id(user_id_str: &str) -> Result<UserId> {
    match user_id_str.parse::<u64>() {
        Ok(user_id_num) => Ok(UserId::new(user_id_num)),
        Err(e) => Err(anyhow!("Failed to parse user ID: {}", e)),
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
    let bot_id_string = secrets
        .get("BOT_USER_ID")
        .expect("Expected a Bot ID in the environment");
    let openai_api_key = secrets
        .get("OPENAI_API_KEY")
        .expect("Expected an OpenAI API key in the environment");

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let serenity_client = Client::builder(&token, intents)
        .event_handler(Handler {
            bot_id: string_to_user_id(&bot_id_string)?,
            openai_client: ReqwestClient::new(),
            openai_api_key,
        })
        .await
        .expect("Err creating client");

    Ok(Gordon {
        client: serenity_client,
    })
}
