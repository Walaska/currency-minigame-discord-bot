use std::fs::File;
use std::io::{self, BufRead, BufReader};
use chrono::{Duration, Utc, TimeZone};
use serenity::{
    framework::standard::{macros::command, Args, CommandResult,},
    model::prelude::*,
    prelude::*,
};
use rand::{Rng, SeedableRng, rngs::StdRng};
use mongodb::{bson::{doc, Document,  DateTime}, Collection, Database};
use crate::MongoDb;

// Daily currency numbas
const DAILY_CURRENCY_LOWER: u32 = 30;
const DAILY_CURRENCY_UPPER: u32 = 80;
// Daily streak growth
const DAILY_STREAK_GROWTH_FACTOR: f64 = 1.008;

#[command]
async fn daily(ctx: &Context, msg: &Message, mut _args: Args) -> CommandResult {
    let client = {
        let data_read = ctx.data.read().await;
        data_read.get::<MongoDb>().expect("Expected MongoDb").clone()
    };
    let db = client.database("currency");
    let collection = db.collection::<Document>("user_cooldowns");
    let filter = doc! {"user_id": msg.author.id.get().clone() as i64};
    if let Some(document) = collection.find_one(filter, None).await? {
        if let Ok(cooldown) = document.get_datetime("daily_cooldown") {
            let naive_cooldown = chrono::NaiveDateTime::from_timestamp_opt(
                cooldown.timestamp_millis() / 1000,
                (cooldown.timestamp_millis() % 1000) as u32 * 1_000_000,
            ).unwrap();
            let cooldown_datetime = Utc.from_utc_datetime(&naive_cooldown);
            let current_time = Utc::now();
            let time_difference = current_time - cooldown_datetime;
            let one_day = Duration::days(1);

            if time_difference >= one_day {
                let user_doc = doc! {
                    "user_id": msg.author.id.get().clone() as i64
                };
                let cooldown_update_doc = doc! {
                    "$set": {
                        "daily_cooldown": DateTime::now()
                    }
                };
                if let Err(e) = collection.update_one(user_doc, cooldown_update_doc, None).await {
                    eprintln!("{:?}", e);
                }
                msg.reply(&ctx, calculate_streak(document, db, msg.author.id).await).await?;
                return Ok(());
            } else {
                let remaining_time = one_day - time_difference;
                msg.reply(&ctx, format!("You're still on cooldown. Time remaining -> {}", format_duration(remaining_time))).await?;
                return Ok(());
            }
        }
    
    }
    let user_doc = doc! {
        "user_id": msg.author.id.get().clone() as i64
    };
    let cooldown_update_doc = doc! {
        "$set": {
            "daily_cooldown": DateTime::now()
        }
    };
    let options = mongodb::options::UpdateOptions::builder().upsert(true).build();
    collection.update_one(user_doc, cooldown_update_doc, options).await?;

    let random_number = calculate_currency();
    let filter = doc! {"user_id": msg.author.id.get().clone() as i64};
    let options = mongodb::options::UpdateOptions::builder().upsert(true).build();
    let update_doc = doc! {"$inc": {"currency": random_number}};
    let collection = db.collection::<Document>("user_currency");
    if let Err(e) = collection.update_one(filter, update_doc, options).await {eprintln!("{:?}", e)}
    msg.reply(&ctx, format!("## WELCOME TO `.daily`! <a:wiggle:1021062305213071440>\n\nKeep coming back every day for more coins the longer you have your <a:fire:1044851524326653994> **streak**.\nTo see what you can do with your coins check out `.shop`!\n*hint: we have a lot of things planned*\n\nAnyway, here's your **{}** <a:DAcoin2:1137457024729370754> coins!", random_number)).await?;
    
    Ok(())
}

fn format_duration(duration: Duration) -> String {
    let hours = duration.num_hours();
    let minutes = duration.num_minutes() % 60;
    format!("**{}** hours and **{}** minutes", hours, minutes)
}

async fn calculate_streak(document: Document, db: Database, user_id: UserId) -> String {
    let two_days_ago = Utc::now() - Duration::days(2);
    if let Ok(last_daily) = document.get_datetime("daily_cooldown") {
        let naive_last_daily = chrono::NaiveDateTime::from_timestamp_opt(
            last_daily.timestamp_millis() / 1000,
            (last_daily.timestamp_millis() % 1000) as u32 * 1_000_000,
        ).unwrap();
        let last_daily_datetime = Utc.from_utc_datetime(&naive_last_daily);
        let update_filter: Document; 
        if last_daily_datetime <= two_days_ago {
            update_filter = doc! {"$set": {"daily_streak": 0}};
        } else {
            update_filter = doc! {"$inc": {"daily_streak": 1}};
        }
        let filter = doc! {"user_id": user_id.get().clone() as i64};
        let options = mongodb::options::UpdateOptions::builder().upsert(true).build();

        let collection = db.collection::<Document>("user_currency");
        if let Err(e) = collection.update_one(filter, update_filter, options).await {eprintln!("{:?}", e)}

        return check_currency(collection, &user_id.get()).await;
    }
    format!("Something went terribly wrong...")
}

async fn check_currency(collection: Collection<Document>, user_id: &u64) -> String {
    let filter = doc! {"user_id": user_id.clone() as i64};
    if let Some(document) = collection.find_one(filter.clone(), None).await.unwrap() {
        if let Ok(daily_streak) = document.get_i32("daily_streak") {
            if daily_streak == 0 {
                let random_number = calculate_currency();
                let update_doc = doc! {"$inc": {"currency": random_number}};
                let options = mongodb::options::UpdateOptions::builder().upsert(true).build();
                collection.update_one(filter, update_doc, options).await.unwrap();
                zero_streak_message(random_number)
            } else {
                let seed: [u8; 32] = rand::random();
                let mut rng = StdRng::from_seed(seed);
                let random_number: i64 = rng.gen_range(calculate_increased_rate(DAILY_CURRENCY_LOWER as f64, daily_streak)..=calculate_increased_rate(DAILY_CURRENCY_UPPER as f64, daily_streak));
                println!("Coin range {} - {}", calculate_increased_rate(DAILY_CURRENCY_LOWER as f64, daily_streak), calculate_increased_rate(DAILY_CURRENCY_UPPER as f64, daily_streak));
                let update_doc = doc! {"$inc": {"currency": random_number}};
                let options = mongodb::options::UpdateOptions::builder().upsert(true).build();
                collection.update_one(filter, update_doc, options).await.unwrap();
                return daily_message(random_number, daily_streak);
            }
        } else {
            // If streak is 0 or not found
            let random_number = calculate_currency();
            let update_doc = doc! {"$inc": {"currency": random_number}};
            let options = mongodb::options::UpdateOptions::builder().upsert(true).build();
            collection.update_one(filter, update_doc, options).await.unwrap();
            format!("Nice, you gained {} coins", random_number as u64)
        }
    } else {
        let random_number = calculate_currency();
        let insert_doc = doc! {"user_id": user_id.clone() as i64, "daily_streak": 0, "currency": random_number};
        collection.insert_one(insert_doc, None).await.unwrap();
        format!("Nice, you gained **{}** coins! (Btw this is a rare message and you should probably let someone know you got this :))", random_number)
    }
}

fn zero_streak_message(coins: i64) -> String {
    let file = File::open("zero_streak_daily_messages.txt").expect("Can't find the file...");
    let reader = BufReader::new(file);

    let mut messages = Vec::new();

    for line in reader.lines() {
        if let Ok(line_content) = line {
            messages.push(line_content);
        }
    }

    let seed: [u8; 32] = rand::random();
    let mut rng = StdRng::from_seed(seed);
    let random_number: usize = rng.gen_range(0..messages.len());

    let message = match messages.get(random_number) {
        Some(message) => message,
        None => return "No messages found".to_string(),
    };

    message
        .replace("{coins}", &coins.to_string())
        .replace(r"\n", "\n")
}

fn daily_message(coins: i64, streak: i32) -> String {
    let mut messages = load_messages_from_file("streak_daily_messages.txt", streak).unwrap_or_else(|err| {
        eprintln!("Error loading messages: {:?}", err);
        Vec::new()
    });

    if messages.is_empty() {
        messages = load_random_messages().unwrap_or_else(|err| {
            eprintln!("Error loading messages: {:?}", err);
            Vec::new()
        });
    }

    let seed: [u8; 32] = rand::random();
    let mut rng = StdRng::from_seed(seed);
    let random_number: usize = rng.gen_range(0..messages.len());

    let message = match messages.get(random_number) {
        Some(message) => message,
        None => return "No messages found".to_string(),
    };

    message
        .replace("{coins}", &coins.to_string())
        .replace("{streak}", &streak.to_string())
        .replace(r"\n", "\n")
}

fn load_random_messages() -> io::Result<Vec<String>> {
    let file = File::open("random_daily_messages.txt")?;
    let reader = BufReader::new(file);

    let mut messages = Vec::new();

    for line in reader.lines() {
        if let Ok(line_content) = line {
            if &line_content != "" {
                messages.push(line_content);
            }
        }
    }

    Ok(messages)
}

fn load_messages_from_file(filename: &str, streak: i32) -> io::Result<Vec<String>> {
    let mut to_capture = false;
    let file = File::open(filename)?;
    let reader = BufReader::new(file);

    let mut messages = Vec::new();

    for line in reader.lines() {
        if let Ok(line_content) = line {
            if &line_content == &format!("day {}", streak) {
                to_capture = true;
                continue;
            }
            if &line_content == "<<EOF" {
                to_capture = false;
            }
            if to_capture {
                messages.push(line_content);
            }
        }
    }

    Ok(messages)
}

fn calculate_currency() -> i64 {
    let seed: [u8; 32] = rand::random();
    let mut rng = StdRng::from_seed(seed);
    let random_number: f64 = rng.gen_range(DAILY_CURRENCY_LOWER..=DAILY_CURRENCY_UPPER) as f64;
    random_number as i64
}

fn calculate_increased_rate(base_rate: f64, streak: i32) -> i64 {
    let mut increased_rate = base_rate;
    
    for _ in 0..streak {
        increased_rate *= DAILY_STREAK_GROWTH_FACTOR;
    }

    increased_rate as i64
}