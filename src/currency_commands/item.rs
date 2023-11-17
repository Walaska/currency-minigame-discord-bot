use std::time::Duration;

use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::*,
    prelude::*,
};
use mongodb::{bson::{doc, Document}, Database};
use crate::MongoDb;
use crate::currency_commands::shop_items::cloak::cloak;

#[command]
async fn item(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let item_str = match args.single::<String>() {
        Ok(arg) => arg,
        Err(_) => {
            msg.reply(&ctx, "To use an item, type `.item <item>`!\nCheck what items you are able to use from your inventory `.inv` :magic_wand:").await?;
            return Ok(());
        }
    };

    match item_str.to_lowercase().as_str() {
        "cloak" | "1" => use_cloak(&ctx, &msg, args).await,
        "uwuify" | "17" => use_uwuify(&ctx, &msg, args).await,
        "steve" | "2" => use_skin(&ctx, &msg, "<a:steve:1153241655944675418>").await,
        "riley" | "3" => use_skin(&ctx, &msg, "<a:riley:1153241629931610132>").await,
        "angry riley" | "4" => use_skin(&ctx, &msg, "<a:angryR:1153241416017924116>").await,
        "choco trex" | "5" => use_skin(&ctx, &msg, "<a:Choctrex:1153241426688229406>").await,
        "durian steve" | "6" => use_skin(&ctx, &msg, "<a:duriansteve:1153241508854648863>").await,
        "cat fish" | "7" => use_skin(&ctx, &msg, "<a:fish:1153241519231356978>").await,
        "riley potter" | "8" => use_skin(&ctx, &msg, "<a:Rileywiz:1153241649900687420>").await,
        "trex builder" | "9" => use_skin(&ctx, &msg, "<a:maintanance:1153241526562996314>").await,
        "turtle steve" | "10" => use_skin(&ctx, &msg, "<a:turtlesteve:1153241671648170035>").await,
        "alien fish" | "11" => use_skin(&ctx, &msg, "<a:alienfish:1153241407453143090>").await,
        "bags" | "12" => use_skin(&ctx, &msg, "<a:bags:1157977401616248842>").await,
        "cool fish" | "13" => use_skin(&ctx, &msg, "<a:coolfish:1153241497278369812>").await,
        "socket" | "14" => use_skin(&ctx, &msg, "<a:socket:1157977422340296748>").await,
        "free art" | "15" => use_skin(&ctx, &msg, "<:NoFreeArt:1002717330838663239>").await,
        "kings" | "16" => use_skin(&ctx, &msg, "<a:king:1157977404015378432>").await,
        "neyas secret private emote lol" | "3230563983" => use_skin(&ctx, &msg, "<a:wiggle:1021062305213071440>").await,
        "walas secret private emote lol" | "69694206969" => use_skin(&ctx, &msg, "<:walas:1138145839668273173>").await,
        "secretnuggie" | "secretnuggie" => use_skin(&ctx, &msg, "<a:FoundEasterEgDotBuySECRETNUGGIE:1157909420152471634>").await,

        _ => {
            return Ok(());
        }
    };
    Ok(())
}

async fn use_uwuify(ctx: &Context, msg: &Message, mut args: Args) {
    let user_id = match args.single::<UserId>() {
        Ok(arg) => arg,
        Err(e) => {
            println!("{}", e);
            msg.reply(&ctx, "<:hihi:1048940632963559444> You need to pick someone to condemn with UwUs. ").await.expect("Error sending message");
            return;
        }
    };

    let member = match ctx.http.get_member(msg.guild_id.unwrap(), user_id).await {
        Ok(member) => member,
        Err(_) => return
    };
    let client = {
        let data_read = ctx.data.read().await;
        data_read.get::<MongoDb>().expect("Expected MongoDb").clone()
    };
    let db = client.database("currency");
    let collection = db.collection::<Document>("user_inventory");

    let filter = doc! {"user_id": msg.author.id.get().clone() as i64};
    if let Some(document) = collection.find_one(filter.clone(), None).await.expect("I couldn't find that person, and I looked far and wide. **Source: trust me**") {
        if let Ok(uwuify) = document.get_i64("uwuify") {
            if uwuify >= 1 as i64 {
                let options = mongodb::options::UpdateOptions::builder().upsert(true).build();
                let update_doc = doc! {"$inc": {"uwuify": -1}};
                collection.update_one(filter.clone(), update_doc, options.clone()).await.unwrap();

                let _ = msg.reply(ctx, format!("<:uwuyes:1159202322950406204> {} is now uwuified uwu", member.display_name())).await;
                let collection = db.collection::<Document>("user_modifiers");
                let options = mongodb::options::UpdateOptions::builder().upsert(true).build();
                let update_doc = doc! {"$set": {"uwuify": true}};
                collection.update_one(filter.clone(), update_doc, options.clone()).await.unwrap();
                tokio::time::sleep(Duration::from_secs(180)).await;
                let update_doc = doc! {"$set": {"uwuify": false}};
                collection.update_one(filter, update_doc, options.clone()).await.unwrap();
                return;
            }
        }
    }
    let _ = msg.reply(ctx, "How are you gonna **uwuify** someone when you don't have any? Better buy some. <:walas3:996796171005743265>\nUse `.buy uwuify`").await;
}

async fn use_skin(ctx: &Context, msg: &Message, skin_to_use: &str) {
    let client = {
        let data_read = ctx.data.read().await;
        data_read.get::<MongoDb>().expect("Expected MongoDb").clone()
    };
    let db = client.database("currency");
    let collection = db.collection::<Document>("user_inventory");

    let filter = doc! {"user_id": msg.author.id.get().clone() as i64};
    if let Some(document) = collection.find_one(filter.clone(), None).await.expect("I couldn't find that person, and I looked far and wide. **Source: trust me**") {
        if let Ok(skins) = document.get_array("owned_skins") {
            for skin in skins {
                if skin.as_str().unwrap() == skin_to_use {
                    let options = mongodb::options::UpdateOptions::builder().upsert(true).build();
                    let update_doc = doc! {"$set": {"dinorace_skin": skin_to_use}};
                    collection.update_one(filter.clone(), update_doc, options.clone()).await.unwrap();
                    let _ = msg.reply(ctx, format!("You've equipped the {} skin. <:hehe:1048940632963559444>", skin)).await;
                    return;
                }
            }
            let _ = msg.reply(ctx, "You don't own that skin").await;
            return;
        }
    }
    let _ = msg.reply(ctx, "You don't own that skin").await;
}

async fn use_cloak(ctx: &Context, msg: &Message, mut args: Args) {
    let user_id = match args.single::<UserId>() {
        Ok(arg) => arg,
        Err(_) => {
            msg.reply(&ctx, "<:hihi:1048940632963559444> You need to mention someone to use an item on. Do it!").await.expect("Error sending message");
            return;
        }
    };

    let member = match ctx.http.get_member(msg.guild_id.unwrap(), user_id).await {
        Ok(member) => member,
        Err(_) => return
    };
    let client = {
        let data_read = ctx.data.read().await;
        data_read.get::<MongoDb>().expect("Expected MongoDb").clone()
    };
    let db = client.database("currency");
    let collection = db.collection::<Document>("user_inventory");

    let filter = doc! {"user_id": msg.author.id.get().clone() as i64};
    if let Some(document) = collection.find_one(filter.clone(), None).await.expect("I couldn't find that person, and I looked far and wide. **Source: trust me**") {
        if let Ok(cloaks) = document.get_i64("cloak") {
            if cloaks >= 1 as i64 {
                if check_cloak_cd(db.clone(), &member).await {
                    let _ = msg.reply(ctx, "This user is cloaked already, can't you see them? *haha..* <:ehe:1087486036189393016>").await;
                    return;
                }
                let options = mongodb::options::UpdateOptions::builder().upsert(true).build();
                let update_doc = doc! {"$inc": {"cloak": -1}};
                collection.update_one(filter.clone(), update_doc, options.clone()).await.unwrap();
                update_cloak_cd(db.clone(), &member, true).await;

                let _ = msg.reply(ctx, format!("<a:spooky:1136720506805043341> YOU'VE CLOAKED **{}**. *poof*\nᴼᴴ ᴺᴼ ᵂᴴᴱᴿᴱ ᴰᴵᴰ ᴱᴹ ᴳᴼ", member.display_name())).await;
                cloak(&ctx, msg.guild_id.unwrap(), member.user.id).await;
                update_cloak_cd(db.clone(), &member, false).await;
                return;
            }
        }
    }
    let _ = msg.reply(ctx, "How are you gonna **cloak** someone when you don't have any? Better buy some. <:walas3:996796171005743265>\nUse `.buy cloak`").await;
}

async fn check_cloak_cd(db: Database, member: &Member) -> bool {
    let collection = db.collection::<Document>("user_cooldowns");
    let filter = doc! {"user_id": member.user.id.get().clone() as i64};
    if let Some(document) = collection.find_one(filter.clone(), None).await.expect("User not found") {
        if let Ok(cloak_cd) = document.get_bool("cloak_cooldown") {
            return cloak_cd;
        }
    }
    false
}

async fn update_cloak_cd(db: Database, member: &Member, cloak_cd: bool) {
    let collection = db.collection::<Document>("user_cooldowns");
    let filter = doc! {"user_id": member.user.id.get().clone() as i64};
    let options = mongodb::options::UpdateOptions::builder().upsert(true).build();
    let update_doc = doc! {"$set": {"cloak_cooldown": cloak_cd}};
    collection.update_one(filter.clone(), update_doc, options.clone()).await.unwrap();
}