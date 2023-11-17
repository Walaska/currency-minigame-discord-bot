use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::*,
    prelude::*, builder::EditMessage,
};
use rand::rngs::StdRng;
use rand::SeedableRng;
use rand::Rng;
use tokio::time::sleep;
use std::time::Duration;
use mongodb::{bson::{doc, Document}, Collection};
use crate::MongoDb;

#[command]
async fn flip(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let user_choice = match args.single::<String>().unwrap_or_default().to_lowercase().as_str() {
        "head" | "heads" => "<:heads:1137444232970391552> Heads",
        "tail" | "tails" => "<:tails:1137444234551631933> Tails",
        _ => {
            msg.channel_id.say(&ctx.http, "What are you trying to flip? This coin? <a:verticalflip:1137452294489780284>\nUse `.flip <heads/tails> <amount>`").await?;
            return Ok(());
        }
    };

    let bet_amount = match args.single::<u32>() {
        Ok(amount) if amount > 0 => amount,
        _ => {
            msg.channel_id.say(&ctx.http, "Invalid bet amount. Please provide a positive number.").await?;
            return Ok(());
        }
    };

    let client = {
        let data_read = ctx.data.read().await;
        data_read.get::<MongoDb>().expect("Expected MongoDb").clone()
    };
    let db = client.database("currency");
    let collection = db.collection::<Document>("user_currency");

    let filter = doc! {"user_id": msg.author.id.get().clone() as i64};
    if let Some(document) = collection.find_one(filter.clone(), None).await? {
        if let Ok(currency) = document.get_i64("currency") {
            if currency >= bet_amount as i64 {
                play_flip(ctx, msg, collection, bet_amount as i64, user_choice).await?;
                return Ok(());
            } else {
                msg.reply(&ctx, format!("*checks wallet*, yep.. you're gonna need a lil more to place that bet. You have: **{}**", currency)).await?;
                return Ok(());
            }
        }
    }

    msg.reply(&ctx, "*checks wallet*, yep.. you're gonna need a lil more to place that bet. You have: **0**").await?;
    Ok(())
}

async fn play_flip(ctx: &Context, msg: &Message, collection: Collection<Document>, bet_amount: i64, user_choice: &str) -> CommandResult {
    let filter = doc! {"user_id": msg.author.id.get().clone() as i64};
    let options = mongodb::options::UpdateOptions::builder().upsert(true).build();
    let update_doc = doc! {"$inc": {"currency": (bet_amount as i64 * -1)}};
    collection.update_one(filter.clone(), update_doc, options).await.unwrap();

    let seed: [u8; 32] = rand::random();
    let mut rng = StdRng::from_seed(seed);
    let coin_result = if rng.gen::<bool>() { "<:heads:1137444232970391552> Heads" } else { "<:tails:1137444234551631933> Tails" };

    let mut message = msg.channel_id.say(&ctx.http, "Flippin da coin <a:flip:1137431367727198251>").await?;
    sleep(Duration::from_secs(1)).await;

    let mut result_message = format!("Coin flip result:\n**{}**\n", coin_result);
    if coin_result == user_choice {
        let winnings = bet_amount as f32 * 1.5;
        result_message.push_str(&format!("Congratulations! You won **{}**<a:DAcoin2:1137457024729370754> coins.", winnings as i64));

        let update_doc = doc! {"$inc": {"currency": winnings as i64}};
        let options = mongodb::options::UpdateOptions::builder().upsert(true).build();
        collection.update_one(filter, update_doc, options.clone()).await.unwrap();
    } else {
        result_message.push_str("You lost! But you **won** in my heart.");
    }

    let builder = EditMessage::new().content(result_message);
    message.edit(&ctx.http, builder).await?;
    Ok(())
}