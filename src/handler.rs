use crate::openai::{OpenAIClient, OpenAIMessage};
use anyhow::{anyhow, Result};
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::prelude::UserId;
use serenity::prelude::{Context, EventHandler};

pub struct Handler {
    pub bot_id: UserId,
    pub openai_client: OpenAIClient,
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.mentions_user_id(self.bot_id) {
            let response = self.get_openai_response(&msg).await;

            match response {
                Ok(reply) => {
                    if let Err(why) = msg.channel_id.say(&ctx.http, reply).await {
                        eprintln!("Error sending message: {:?}", why);
                    }
                }
                Err(err) => {
                    eprintln!("Error getting OpenAI response: {:?}", err);
                }
            }
        } else if msg.content == "!ping" {
            if let Err(why) = msg.channel_id.say(&ctx.http, "pong").await {
                eprintln!("Error sending message: {:?}", why);
            }
        }
    }

    async fn ready(&self, _ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

impl Handler {
    pub async fn get_openai_response(&self, msg: &Message) -> Result<String> {
        let openai_message = OpenAIMessage::new("user", &msg.content);
        self.openai_client.ask(vec![openai_message]).await
    }
}

pub fn string_to_user_id(user_id_str: &str) -> Result<UserId> {
    user_id_str
        .parse::<u64>()
        .map(UserId::new)
        .map_err(|e| anyhow!("Failed to parse user ID: {}", e))
}
