use serenity::{
    framework::standard::{macros::command, Args, CommandResult,},
    model::prelude::*,
    prelude::*,
    builder::{CreateEmbed, CreateMessage, EditMessage, CreateEmbedFooter, CreateEmbedAuthor},
};
use rand::{Rng, SeedableRng, rngs::StdRng};
use tokio::time::sleep;
use std::time::Duration;
use mongodb::{bson::{doc, Document}, Collection};
use crate::MongoDb;

#[command]
#[bucket = "slots"]
async fn slots(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let bet_amount = match args.single::<String>() {
        Ok(arg) => {
            match arg.to_lowercase().as_str() {
                "max" | "all" => {
                    handle_max_bet(ctx, msg).await
                }
                "half" => {
                    handle_half_bet(ctx, msg).await
                }
                _ => {
                    match arg.parse::<i64>() {
                        Ok(amount) if amount > 0 => amount,
                        _ => {
                            msg.channel_id.say(&ctx.http, "<:hm:1041810538369388554> That's not right, try specifying a number, **max/all** or **half**.").await?;
                            return Ok(());
                        }
                    }
                }
            }
        }
        Err(_) => {
            msg.channel_id.say(&ctx.http, "<:hm:1041810538369388554> That's not right, try specifying a number, **max/all** or **half**.").await?;
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
            if currency >= bet_amount as i64 && bet_amount > 0 {
                play_slots(ctx, msg, collection, bet_amount as i64).await?;
                return Ok(());
            } else {
                msg.reply(&ctx, format!("You're low on money for that bet <:shrug:998328408738115664> Your current is balance is: **{}**", currency)).await?;
                return Ok(());
            }
        }
    }
    msg.reply(&ctx, "You're low on money for that bet <:shrug:998328408738115664> Your current balance is: **0**").await?;

    Ok(())
}

async fn handle_max_bet(ctx: &Context, msg: &Message) -> i64 {
    let client = {
        let data_read = ctx.data.read().await;
        data_read.get::<MongoDb>().expect("Expected MongoDb").clone()
    };
    let db = client.database("currency");
    let collection = db.collection::<Document>("user_currency");

    let filter = doc! {"user_id": msg.author.id.get().clone() as i64};
    if let Some(document) = collection.find_one(filter.clone(), None).await.expect("Can't find user...") {
        if let Ok(currency) = document.get_i64("currency") {
            return currency;
        }
    }
    0
}

async fn handle_half_bet(ctx: &Context, msg: &Message) -> i64 {
    let client = {
        let data_read = ctx.data.read().await;
        data_read.get::<MongoDb>().expect("Expected MongoDb").clone()
    };
    let db = client.database("currency");
    let collection = db.collection::<Document>("user_currency");

    let filter = doc! {"user_id": msg.author.id.get().clone() as i64};
    if let Some(document) = collection.find_one(filter.clone(), None).await.expect("Can't find user...") {
        if let Ok(currency) = document.get_i64("currency") {
            return currency / 2;
        }
    }
    0
}

// Helper function to choose a random index based on probabilities
fn select_random_index(probabilities: &[f64], rng: &mut StdRng) -> usize {
    let total_probability: f64 = probabilities.iter().sum();
    let random_value = rng.gen_range(0.0..total_probability);

    let mut cumulative_probability = 0.0;
    for (index, &probability) in probabilities.iter().enumerate() {
        cumulative_probability += probability;
        if random_value < cumulative_probability {
            return index;
        }
    }

    probabilities.len() - 1
}

async fn play_slots(ctx: &Context, msg: &Message, collection: Collection<Document>, bet_amount: i64) -> CommandResult {
    let filter = doc! {"user_id": msg.author.id.get().clone() as i64};
    let options = mongodb::options::UpdateOptions::builder().upsert(true).build();
    let update_doc = doc! {"$inc": {"currency": (bet_amount as i64 * -1)}};
    collection.update_one(filter.clone(), update_doc, options).await.unwrap();

    let emojis = vec!["🍎", "🍊", "🍋", "🍒", "🍇", "🍉"];
    let probabilities = vec![0.1666666667, 0.1666666667, 0.1666666667, 0.1666666667, 0.1666666667, 0.1666666667];
    let mut rng = StdRng::from_entropy();
    let mut selected_emojis: Vec<&str> = vec![];

    for _ in 0..3 {
        let random_index = select_random_index(&probabilities, &mut rng);
        selected_emojis.push(emojis[random_index]);
    }

    let member = ctx.http.get_member(msg.guild_id.unwrap(), msg.author.id).await?;

    let author = CreateEmbedAuthor::new(format!("{}'s Slot Game", member.display_name())).icon_url(member.face());
    let footer = CreateEmbedFooter::new("Match two identical symbols adjacent to each other for a win, but to hit the jackpot, aim for three consecutive symbols!\n\nCommand to play -> .slots <bet amount>");
    let embed = CreateEmbed::default().title("Slots")
    .author(author.clone())
    .description("<a:slot:1135268042121695493> <a:slot:1135268042121695493> <a:slot:1135268042121695493>")
        .color(Color::from_rgb(255, 134, 134))
    .footer(footer);

    let builder = CreateMessage::new().add_embed(embed);
    let mut sent_message = msg.channel_id.send_message(&ctx.http, builder).await?;

    let mut win_count = 0;
    for i in 0..2 {
        if selected_emojis[i] == selected_emojis[i + 1] {
            win_count += 1;
        }
    }

    let winnings = bet_amount * match win_count {
        0 => 0,
        1 => 2,
        2 => 3,
        _ => unreachable!(),
    };

    sleep(Duration::from_secs(1)).await;
    sent_message.edit(ctx, get_slots_embed("Slots".to_string(), format!("{} <a:slot:1135268042121695493> <a:slot:1135268042121695493>", selected_emojis[0]), &author)).await.expect("Something went wrong when trying to edit embed...");

    sleep(Duration::from_secs(1)).await;
    sent_message.edit(ctx, get_slots_embed("Slots".to_string(), format!("{} {} <a:slot:1135268042121695493>", selected_emojis[0], selected_emojis[1]), &author)).await.expect("Something went wrong when trying to edit embed...");

    sleep(Duration::from_secs(1)).await;

    if win_count > 0 {
        let update_doc = doc! {"$inc": {"currency": winnings as i64}};
        let options = mongodb::options::UpdateOptions::builder().upsert(true).build();
        collection.update_one(filter, update_doc, options.clone()).await.unwrap();
        let footer = CreateEmbedFooter::new("Match two identical symbols adjacent to each other for a win, but to hit the jackpot, aim for three consecutive symbols!\n\nCommand to play -> .slots <bet amount>");
        let builder = EditMessage::new().add_embed(CreateEmbed::default()
        .title("Slots")
        .author(author.clone())
        .description(format!("{} {} {}", selected_emojis[0], selected_emojis[1], selected_emojis[2]))
        .field(format!("{}'s Winnings", member.display_name()), format!("**{}** coins", winnings), false).color(Color::from_rgb(119,221,119))
        .footer(footer));

        sent_message.edit(ctx, builder).await.expect("Something went wrong when trying to edit embed...");
    } else {
        let footer = CreateEmbedFooter::new("Match two identical symbols adjacent to each other for a win, but to hit the jackpot, aim for three consecutive symbols!\n\nCommand to play -> .slots <bet amount>");
        let builder = EditMessage::new().add_embed(CreateEmbed::default()
        .title("Slots")
        .author(author.clone())
        .description(format!("{} {} {}", selected_emojis[0], selected_emojis[1], selected_emojis[2]))
        .field(format!("{}'s Results", member.display_name()), "**You didn't win anything :(**\n\n", false).color(Color::from_rgb(255, 105, 97))
        .footer(footer));

        sent_message.edit(ctx, builder).await.expect("Something went wrong when trying to edit embed...");
    }
    Ok(())
}

fn get_slots_embed(title: String, description: String, author: &CreateEmbedAuthor) -> EditMessage {
    let footer = CreateEmbedFooter::new("Match two identical symbols adjacent to each other for a win, but to hit the jackpot, aim for three consecutive symbols!\n\nCommand to play -> .slots <bet amount>");

    EditMessage::new().add_embed(
    CreateEmbed::default()
    .title(title)
    .author(author.clone())
    .description(description)
        .color(Color::from_rgb(50,51,55))
    .footer(footer))
}