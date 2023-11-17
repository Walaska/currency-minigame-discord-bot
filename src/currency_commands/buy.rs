use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::*,
    prelude::*,
};

use mongodb::{bson::{doc, Document}, Database};
use crate::MongoDb;
// Common Skins
const common_price: i64 = 100;
// Uncommon
const uncommon_price: i64 = 1000;
// Rare
const rare_price: i64 = 5000;
// Epic
const epic_price: i64 = 25000;
// Legendary
const legendary_price: i64 = 100000;
// Mythic
const mythic_price: i64 = 1000000;
// easter eggs
const free_price: i64 = 1;
#[command]
async fn buy(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let item_str = match args.single::<String>() {
        Ok(arg) => arg,
        Err(_) => {
            msg.reply(&ctx, "Please provide an item name, like so: `.buy cloak` <:heheyea:1137673283500785694>\nYou can check what's for sale in `.shop`").await?;
            return Ok(());
        }
    };

    let client = {
        let data_read = ctx.data.read().await;
        data_read.get::<MongoDb>().expect("Expected MongoDb").clone()
    };
    let db = client.database("currency");

    let response = match item_str.to_lowercase().as_str() {
        "cloak" | "1" => buy_item(db, &msg, 250, "cloak").await,
        "steve" | "2" => buy_skin(db, &msg, "<a:steve:1153241655944675418>", common_price).await,
        "riley" | "3" => buy_skin(db, &msg, "<a:riley:1153241629931610132>", common_price).await,
        "angry riley" | "4" => buy_skin(db, &msg, "<a:angryR:1153241416017924116>", uncommon_price).await,
        "choco trex" | "5" => buy_skin(db, &msg, "<a:Choctrex:1153241426688229406>", rare_price).await,
        "durian steve" | "6" => buy_skin(db, &msg, "<a:duriansteve:1153241508854648863>", rare_price).await,
        "cat fish" | "7" => buy_skin(db, &msg, "<a:fish:1153241519231356978>", rare_price).await,
        "riley potter" | "8" => buy_skin(db, &msg, "<a:Rileywiz:1153241649900687420>", epic_price).await,
        "trex builder" | "9" => buy_skin(db, &msg, "<a:maintanance:1153241526562996314>", epic_price).await,
        "turtle steve" | "10" => buy_skin(db, &msg, "<a:turtlesteve:1153241671648170035>", epic_price).await,
        "alien fish" | "11" => buy_skin(db, &msg, "<a:alienfish:1153241407453143090>", epic_price).await,
        "bags" | "12" => buy_skin(db, &msg, "<a:bags:1157977401616248842>", epic_price).await,
        "cool fish" | "13" => buy_skin(db, &msg, "<a:coolfish:1153241497278369812>", legendary_price).await,
        "socket" | "14" => buy_skin(db, &msg, "<a:socket:1157977422340296748>", legendary_price).await,
        "free art" | "15" => buy_skin(db, &msg, "<:NoFreeArt:1002717330838663239>", mythic_price).await,
        "kings" | "16" => buy_skin(db, &msg, "<a:king:1157977404015378432>", mythic_price).await,
        "uwuify" | "17" => buy_item(db, &msg, 500, "uwuify").await,
        "neyas secret private emote lol" | "3230563983" => buy_skin(db, &msg, "<a:wiggle:1021062305213071440>", free_price).await,
        "walas secret private emote lol" | "69694206969" => buy_skin(db, &msg, "<:walas:1138145839668273173>", free_price).await,
        "secretnuggie" | "secretnuggie" => buy_skin(db, &msg, "<a:FoundEasterEgDotBuySECRETNUGGIE:1157909420152471634>", free_price).await,

    _ => {
            "Please provide an item name, like so: `.buy cloak` <:heheyea:1137673283500785694>\nYou can check what's for sale in `.shop` <a:y_sparkle:1086040993326977054>".to_string()
        }
    };

    msg.reply(&ctx, response).await?;
    Ok(())
}

async fn buy_item(db: Database, msg: &Message, price: i64, item: &str) -> String {
            let collection = db.collection::<Document>("user_currency");
        
            let filter = doc! {"user_id": msg.author.id.get().clone() as i64};
            if let Some(document) = collection.find_one(filter.clone(), None).await.expect("User not found") {
                if let Ok(currency) = document.get_i64("currency") {
                    if currency >= price as i64 {
                        let options = mongodb::options::UpdateOptions::builder().upsert(true).build();
                        let update_doc = doc! {"$inc": {"currency": (price as i64 * -1)}};
                        collection.update_one(filter.clone(), update_doc, options.clone()).await.unwrap();
        
                        let collection = db.collection::<Document>("user_inventory");
                        let update_doc = doc! {"$inc": {item: 1 as i64}};
                        collection.update_one(filter.clone(), update_doc, options).await.unwrap();
                        return format!("NICE! <:heheyea:1137673283500785694> This **{}** is now *yours*, use it with `.item {} <@user>`", item, item);
                    }
                }
            }
        
            format!("I'm afraid you need more coins to buy {}. <:huhu:1048941068281978950>", item)
}

async fn buy_skin(db: Database, msg: &Message, skin: &str, skin_price: i64) -> String {
    let collection = db.collection::<Document>("user_currency");

    let filter = doc! {"user_id": msg.author.id.get().clone() as i64};
    if let Some(document) = collection.find_one(filter.clone(), None).await.expect("User not found") {
        if let Ok(currency) = document.get_i64("currency") {
            if currency >= skin_price {
                let options = mongodb::options::UpdateOptions::builder().upsert(true).build();
                let update_doc = doc! {"$inc": {"currency": (skin_price * -1)}};
                collection.update_one(filter.clone(), update_doc, options.clone()).await.unwrap();

                let collection = db.collection::<Document>("user_inventory");
                let update_doc = doc! {"$addToSet": {"owned_skins": skin}};
                collection.update_one(filter.clone(), update_doc, options).await.unwrap();
                return format!("WOW, you've <a:y_sparkle:1086040993326977054>**ACQUIRED**<a:y_sparkle:1086040993326977054> a {} *skin*. Equip it with `.item <number>`\nFor example: `.item 3`!", skin);
            }
        }
    }
    format!("I'm afraid you need more coins to buy that. <:huhu:1048941068281978950>")
}