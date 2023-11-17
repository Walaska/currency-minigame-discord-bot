use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::*,
    prelude::*,
};

use mongodb::bson::{doc, Document};
use crate::MongoDb;

#[command]
async fn coins(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    match args.single::<UserId>() {
        Ok(arg) => other_coins(&ctx, &msg, arg).await?,
        Err(_) => {
            own_coins(&ctx, &msg).await?;
        }
    };
    Ok(())
}

async fn own_coins(ctx: &Context, msg: &Message) -> CommandResult {
    let client = {
        let data_read = ctx.data.read().await;
        data_read.get::<MongoDb>().expect("Expected MongoDb").clone()
    };
    let db = client.database("currency");
    let collection = db.collection::<Document>("user_currency");

    let filter = doc! {"user_id": msg.author.id.get().clone() as i64};
    if let Some(document) = collection.find_one(filter, None).await? {
        if let Ok(currency) = document.get_i64("currency") {
            msg.reply(&ctx, format!("You have **{}** <a:DAcoin2:1137457024729370754> coins!", currency)).await?;
            return Ok(());
        }
    }
    msg.reply(&ctx, format!("You have **0** <a:DAcoin2:1137457024729370754> coins!")).await?;
    Ok(())
}

async fn other_coins(ctx: &Context, msg: &Message, user_id: UserId) -> CommandResult {
    let client = {
        let data_read = ctx.data.read().await;
        data_read.get::<MongoDb>().expect("Expected MongoDb").clone()
    };
    let db = client.database("currency");
    let collection = db.collection::<Document>("user_currency");

    let member = ctx.http.get_member(msg.guild_id.unwrap(), user_id).await?;

    let filter = doc! {"user_id": user_id.get().clone() as i64};
    if let Some(document) = collection.find_one(filter, None).await? {
        if let Ok(currency) = document.get_i64("currency") {
            if let Ok(streak) = document.get_i32("streak") {
                if streak > 10 {
                    msg.reply(&ctx, format!("{} has **{}** <a:DAcoin2:1137457024729370754> coins and an amazing streak of <a:fire:1044851524326653994> **{}**!! Wow..", member.display_name(), currency, streak)).await?;
                    return Ok(());
                }
            }
            msg.reply(&ctx, format!("{} has **{}** <a:DAcoin2:1137457024729370754> coins!", member.display_name(), currency)).await?;
            return Ok(());
        }
    }
    msg.reply(&ctx, format!("{} has **0** <a:DAcoin2:1137457024729370754> coins!", member.display_name())).await?;
    Ok(())
}