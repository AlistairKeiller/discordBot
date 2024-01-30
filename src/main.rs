use cosmic_text::{
    fontdb::Source, Attrs, Buffer, Color, Family, FontSystem, Metrics, Shaping, SwashCache,
};
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
        if msg.content.starts_with('!') {
            let mut font_system = FontSystem::new_with_fonts([Source::Binary(
                std::sync::Arc::new(include_bytes!("Monocraft.ttf")),
            )]);
            let mut swash_cache = SwashCache::new();

            let metrics = Metrics::new(24.0, 30.0);

            let mut buffer = Buffer::new(&mut font_system, metrics);
            let mut buffer = buffer.borrow_with(&mut font_system);

            buffer.set_size(800.0, f32::INFINITY);
            buffer.set_text(
                &msg.content[1..],
                Attrs::new().family(Family::Name("Monocraft")),
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

            let mut image =
                ImageBuffer::<Rgba<u8>, Vec<u8>>::new(max_x as u32 + 2, max_y as u32 + 8);

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
            if image
                .write_to(&mut cursor, image::ImageOutputFormat::Png)
                .is_ok()
            {
                let data = cursor.into_inner();

                let attachment = CreateAttachment::bytes(data, "image.png");

                let mut webhook = None;
                if let Ok(webhooks) = msg.channel_id.webhooks(&ctx.http).await {
                    if let Some(first_webhook) = webhooks.first() {
                        webhook = Some(first_webhook.clone());
                    } else if let Ok(new_webhook) = msg
                        .channel_id
                        .create_webhook(&ctx.http, CreateWebhook::new("render_webhook"))
                        .await
                    {
                        println!("new");
                        webhook = Some(new_webhook);
                    }
                }

                if let Some(webhook) = webhook {
                    let mut builder = ExecuteWebhook::new().add_file(attachment);
                    if let Some(avatar_url) = msg.author.avatar_url() {
                        builder = builder.avatar_url(avatar_url);
                    }
                    if let Some(nick) = msg.author_nick(&ctx.http).await {
                        builder = builder.username(nick);
                    } else if let Some(nick) = msg.author.global_name.clone() {
                        builder = builder.username(nick);
                    }
                    webhook
                        .execute(&ctx.http, false, builder)
                        .await
                        .expect("Could not execute webhook.");
                    if let Err(why) = msg.delete(&ctx.http).await {
                        println!("Client error: {why:?}");
                    };
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
    let token = std::env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
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
