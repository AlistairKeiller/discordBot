use serenity::async_trait;
use serenity::builder::*;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::prelude::*;
use serenity::prelude::*;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        print!("test");
        if msg.content == "!ping" {
            print!("test");
            if let Ok(image) = CreateAttachment::path("image.png").await {
                if let Err(why) = msg
                    .channel_id
                    .send_message(&ctx.http, CreateMessage::new().add_file(image))
                    .await
                {
                    println!("Error sending message: {why:?}");
                }
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    let token = "MTIwMTczOTU1NTAyODYxNTE4OA.GD8buA.aT1Gs2S6CZABC-PfIWKbZPiD6uvfcAkc3YX68E";
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
}
