use cosmic_text::{Attrs, Buffer, Color, FontSystem, Metrics, Shaping, SwashCache};
use image::{ImageBuffer, Rgba};
use serenity::async_trait;
use serenity::builder::*;
use serenity::http::Http;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::prelude::*;
use serenity::model::webhook::Webhook;
use serenity::prelude::*;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content.starts_with("!") {
            let mut font_system = FontSystem::new();
            let mut swash_cache = SwashCache::new();

            let metrics = Metrics::new(24.0, 30.0);

            let mut buffer = Buffer::new(&mut font_system, metrics);
            let mut buffer = buffer.borrow_with(&mut font_system);

            buffer.set_size(800.0, f32::INFINITY);
            buffer.set_text(&msg.content[1..], Attrs::new(), Shaping::Advanced);
            buffer.shape_until_scroll(true);

            let text_color = Color::rgb(0xFF, 0xFF, 0xFF); // Black color

            let mut max_x = 0;
            let mut max_y = 0;
            buffer.draw(&mut swash_cache, text_color, |x, y, w, h, _| {
                for i in x..x + w as i32 {
                    for j in y..y + h as i32 {
                        max_x = std::cmp::max(i, max_x);
                        max_y = std::cmp::max(j, max_y);
                    }
                }
            });

            let mut image = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(max_x as u32, max_y as u32 + 7);

            buffer.draw(&mut swash_cache, text_color, |x, y, w, h, color| {
                let color = Rgba([color.r(), color.g(), color.b(), color.a()]);
                for i in x..x + w as i32 {
                    for j in y..y + h as i32 {
                        if i >= 0 && i < image.width() as i32 && j >= 0 && j < image.height() as i32
                        {
                            image.put_pixel(i as u32, j as u32, color);
                        }
                    }
                }
            });

            let mut cursor = std::io::Cursor::new(Vec::new());
            if let Ok(_) = image.write_to(&mut cursor, image::ImageOutputFormat::Png) {
                let data = cursor.into_inner();

                // Create an attachment from the byte stream
                let attachment = CreateAttachment::bytes(data, "image.png");

                // Send the message with the image
                // if let Err(why) = msg
                //     .channel_id
                //     .send_message(&ctx.http, CreateMessage::new().add_file(attachment))
                //     .await
                // {
                //     println!("Error sending message: {why:?}");
                // }

                let webhook = Webhook::from_url(&ctx.http, "https://discord.com/api/webhooks/1201806780682997790/5oiQ9Zyrm2StAW9HrgmqzVf-PjDSJwilH1Jo8oDt6u7xWyk1Rj6MgAENB84lFGKs76ik")
        .await
        .expect("Replace the webhook with your own");

                let mut builder = ExecuteWebhook::new().add_file(attachment);
                if let Some(avatar_url) = msg.author.avatar_url() {
                    builder = builder.avatar_url(avatar_url);
                }
                if let Some(nick) = msg.author_nick(&ctx.http).await {
                    builder = builder.username(nick);
                } else if let Some(nick) = msg.author.global_name {
                    builder = builder.username(nick);
                }
                webhook
                    .execute(&ctx.http, false, builder)
                    .await
                    .expect("Could not execute webhook.");
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
