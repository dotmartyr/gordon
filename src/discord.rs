use crate::openai::{OpenAIClient, OpenAIMessage};
use anyhow::{anyhow, Error, Result};
use serde::Serialize;
use serenity::all::GetMessages;
use serenity::async_trait;
use serenity::model::channel::{Channel, ChannelType, Message};
use serenity::model::gateway::Ready;
use serenity::model::id::{ChannelId, MessageId, UserId};
use serenity::prelude::{Context, EventHandler};

#[derive(Serialize)]
struct ThreadCreationParams {
    name: String,
    auto_archive_duration: Option<u16>,
    rate_limit_per_user: Option<u16>,
}

pub struct Handler {
    pub bot_id: UserId,
    pub openai_client: OpenAIClient,
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.mentions_user_id(self.bot_id) {
            self.process_reply(&ctx, &msg).await;
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
    async fn process_reply(&self, ctx: &Context, msg: &Message) {
        match self.get_response_channel(ctx, msg).await {
            Ok(response_channel_id) => {
                let messages = self.get_channel_messages(ctx, &response_channel_id).await;
                match messages {
                    Ok(messages) => {
                        let response = self.get_openai_response(messages).await;
                        self.send_response(ctx, &response_channel_id, response)
                            .await;
                    }
                    Err(err) => {
                        eprintln!("Error getting channel messages: {:?}", err);
                    }
                }
            }
            Err(err) => {
                eprintln!("Error determining response channel: {:?}", err);
            }
        }
    }

    async fn get_response_channel(&self, ctx: &Context, msg: &Message) -> Result<ChannelId> {
        if let Ok(channel) = msg.channel_id.to_channel(&ctx.http).await {
            match &channel {
                Channel::Guild(channel) => {
                    println!("Channel name: {}", channel.name);
                    match channel.kind {
                        ChannelType::Text => {
                            self.create_thread_from_message(ctx, msg.channel_id, msg.id)
                                .await
                        }
                        ChannelType::PublicThread | ChannelType::PrivateThread => {
                            Ok(msg.channel_id)
                        }
                        _ => {
                            eprintln!(
                                "Guild Channel type: {:#?} - Response not configured",
                                channel
                            );
                            Ok(msg.channel_id)
                        }
                    }
                }
                _ => {
                    eprintln!("Channel type: {:#?} - Response not configured", channel);
                    Ok(msg.channel_id)
                }
            }
        } else {
            Err(anyhow!("Error fetching channel information"))
        }
    }

    async fn create_thread_from_message(
        &self,
        ctx: &Context,
        channel_id: ChannelId,
        message_id: MessageId,
    ) -> Result<ChannelId> {
        let thread_name = format!("Gordon Conversation: {}", message_id);
        let thread_params = ThreadCreationParams {
            name: thread_name,
            auto_archive_duration: None,
            rate_limit_per_user: None,
        };

        match ctx
            .http
            .create_thread_from_message(channel_id, message_id, &thread_params, None)
            .await
        {
            Ok(thread_channel) => {
                println!("Created thread: {:?}", thread_channel.id);
                Ok(thread_channel.id)
            }
            Err(e) => Err(anyhow!("Failed to create thread: {}", e)),
        }
    }

    fn determine_openai_role(&self, author_id: UserId) -> &'static str {
        if author_id == self.bot_id {
            "assistant"
        } else {
            "user"
        }
    }

    async fn get_channel_messages(
        &self,
        ctx: &Context,
        channel_id: &ChannelId,
    ) -> Result<Vec<OpenAIMessage>> {
        let mut messages = channel_id
            .messages(&ctx.http, GetMessages::default().limit(10))
            .await?;
        messages.reverse();

        let mut openai_messages = Vec::new();

        for message in &messages {
            let (content, role) = if message.content.is_empty() {
                if let Some(ref referenced_message) = message.referenced_message {
                    (
                        &referenced_message.content,
                        self.determine_openai_role(referenced_message.author.id),
                    )
                } else {
                    (&message.content, "user")
                }
            } else {
                (
                    &message.content,
                    self.determine_openai_role(message.author.id),
                )
            };

            openai_messages.push(OpenAIMessage::new(role, content));
        }

        Ok(openai_messages)
    }

    pub async fn get_openai_response(&self, messages: Vec<OpenAIMessage>) -> Result<String> {
        self.openai_client.ask(messages).await
    }

    async fn send_response(
        &self,
        ctx: &Context,
        channel_id: &ChannelId,
        response: Result<String, Error>,
    ) {
        match response {
            Ok(reply) => {
                if let Err(why) = channel_id.say(&ctx.http, reply).await {
                    eprintln!("Error sending message: {:#?}", why);
                }
            }
            Err(err) => {
                eprintln!("Error getting OpenAI response: {:#?}", err);
            }
        }
    }
}

pub fn string_to_user_id(user_id_str: &str) -> Result<UserId> {
    user_id_str
        .parse::<u64>()
        .map(UserId::new)
        .map_err(|e| anyhow!("Failed to parse user ID: {}", e))
}
