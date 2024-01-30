use cosmic_text::{Attrs, Buffer, Color, FontSystem, Metrics, Shaping, SwashCache};
use image::{ImageBuffer, Rgba};
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
        if msg.content == "!ping" {
            let mut font_system = FontSystem::new();
            let mut swash_cache = SwashCache::new();

            let metrics = Metrics::new(24.0, 30.0);

            let mut buffer = Buffer::new(&mut font_system, metrics);
            let mut buffer = buffer.borrow_with(&mut font_system);

            buffer.set_size(800.0, f32::INFINITY);
            buffer.set_text(
                "Hello world! This is a test",
                Attrs::new(),
                Shaping::Advanced,
            );
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

            image.save("image.png").unwrap();

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
