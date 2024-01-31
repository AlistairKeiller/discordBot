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
        if msg.content.starts_with('!') || msg.content.starts_with('~') {
            let mut font_system = FontSystem::new_with_fonts([
                Source::Binary(std::sync::Arc::new(include_bytes!("Monocraft.ttf"))),
                Source::Binary(std::sync::Arc::new(include_bytes!(
                    "The Doctor Regular.ttf"
                ))),
            ]);
            let mut swash_cache = SwashCache::new();

            let metrics = Metrics::new(
                if msg.content.starts_with('!') {
                    32.0
                } else {
                    64.0
                },
                if msg.content.starts_with('!') {
                    36.0
                } else {
                    72.0
                },
            );

            let mut buffer = Buffer::new(&mut font_system, metrics);
            let mut buffer = buffer.borrow_with(&mut font_system);

            buffer.set_size(800.0, f32::INFINITY);
            buffer.set_text(
                &msg.content[1..],
                if msg.content.starts_with('!') {
                    Attrs::new().family(Family::Name("Monocraft"))
                } else {
                    Attrs::new().family(Family::Name("The Doctor"))
                },
                Shaping::Advanced,
            );
            buffer.shape_until_scroll(true);

            let text_color = Color::rgb(0xFF, 0xFF, 0xFF);

            let mut min_x = i32::MAX;
            let mut max_x = i32::MIN;
            let mut min_y = i32::MAX;
            let mut max_y = i32::MIN;
            buffer.draw(&mut swash_cache, text_color, |x, y, w, h, _| {
                for i in x..x + w as i32 {
                    for j in y..y + h as i32 {
                        min_x = std::cmp::min(i, min_x);
                        max_x = std::cmp::max(i, max_x);
                        min_y = std::cmp::min(j, min_y);
                        max_y = std::cmp::max(j, max_y);
                    }
                }
            });

            let mut image = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(
                (max_x - min_x) as u32 + 2,
                (max_y - min_y) as u32 + 16,
            );

            buffer.draw(&mut swash_cache, text_color, |x, y, w, h, color| {
                let color_a = Rgba([color.r(), color.g(), color.b(), color.a()]);
                for i in x..x + w as i32 {
                    for j in y..y + h as i32 {
                        let color_b =
                            image.get_pixel((i - min_x + 1) as u32, (j - min_y + 8) as u32);

                        let alpha_a = color_a[3] as f32 / 255.0;
                        let alpha_b = color_b[3] as f32 / 255.0;

                        let red = (color_a[0] as f32 * alpha_a)
                            + (color_b[0] as f32 * alpha_b * (1.0 - alpha_a));
                        let green = (color_a[1] as f32 * alpha_a)
                            + (color_b[1] as f32 * alpha_b * (1.0 - alpha_a));
                        let blue = (color_a[2] as f32 * alpha_a)
                            + (color_b[2] as f32 * alpha_b * (1.0 - alpha_a));
                        let alpha = 255.0 * (alpha_a + alpha_b * (1.0 - alpha_a));

                        image.put_pixel(
                            (i - min_x + 1) as u32,
                            (j - min_y + 8) as u32,
                            Rgba([red as u8, green as u8, blue as u8, alpha as u8]),
                        );
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
