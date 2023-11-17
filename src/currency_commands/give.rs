use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::*,
    prelude::*,
};

use mongodb::bson::{doc, Document};
use crate::MongoDb;

#[command]
async fn give(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let first_arg = match args.single::<UserId>() {
        Ok(arg) => arg,
        Err(_) => {
            msg.reply(&ctx, "You've given your coins to the void! Jk, pls mention a `@user` <:heheyea:1137673283500785694>").await?;
            return Ok(());
        }
    };
    let second_arg = match args.single::<u64>() {
        Ok(arg) => arg,
        Err(_) => {
            msg.reply(&ctx, "<:huhu:1048941068281978950> You didn't tell me how many coins to give").await?;
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
    if let Some(document) = collection.find_one(filter, None).await? {
        if let Ok(currency) = document.get_i64("currency") {
            if currency >= second_arg as i64 {
                if msg.author.id == first_arg {msg.reply(&ctx, "Would be nice if that worked, but money doesn't grow on trees- or something. <:skilltissue:1115303522196525189>").await?;} else {            
                    if let Ok(user) = ctx.http.get_member(msg.guild_id.unwrap(), first_arg).await {
                        let filter = doc! {"user_id": first_arg.get().clone() as i64};
                        let update_doc = doc! {"$inc": {"currency": second_arg as i64}};
                        let options = mongodb::options::UpdateOptions::builder().upsert(true).build();
                        collection.update_one(filter, update_doc, options.clone()).await.unwrap();
                        let filter = doc! {"user_id": msg.author.id.get().clone() as i64};
                        let update_doc = doc! {"$inc": {"currency": (second_arg as i64 * -1)}};
                        collection.update_one(filter, update_doc, options).await.unwrap();

                        msg.reply(&ctx, format!("**{}**<a:pe_xcoin:1135641808739778690> coins fed into {}'s wallet.. bank? place.", second_arg, user.display_name())).await?;
                    } else {
                        msg.reply(&ctx, format!("I couldn't find that person, and I looked far and wide. **Source: trust me bro**")).await?;
                    }
                }
                return Ok(());

            } else  {           
                msg.reply(&ctx, format!("<:huhu:1048941068281978950> Idk how to say this but.. you're too poor to give that amount.\nYou currently have **{}**<a:DAcoin2:1137457024729370754>", currency)).await?;
                return Ok(());
            }
        }
    }

    msg.reply(&ctx, format!("<:huhu:1048941068281978950> Idk how to say this but.. you're too poor to give that amount.\nYou currently have **0**<a:DAcoin2:1137457024729370754>")).await?;
    Ok(())
}
