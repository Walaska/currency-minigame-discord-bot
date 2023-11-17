use std::time::Duration;
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::*,
    prelude::*, builder::{CreateEmbed, CreateMessage, CreateButton, CreateInteractionResponseMessage, CreateEmbedFooter, CreateEmbedAuthor}, futures::StreamExt,
};
use mongodb::{bson::{doc, Document}, Collection};

use crate::MongoDb;

#[command]
async fn inv(ctx: &Context, msg: &Message, mut _args: Args) -> CommandResult {
    let client = {
        let data_read = ctx.data.read().await;
        data_read.get::<MongoDb>().expect("Expected MongoDb").clone()
    };
    let db = client.database("currency");
    let collection = db.collection::<Document>("user_inventory");

    let member = match ctx.http.get_member(msg.guild_id.unwrap(), msg.author.id).await {
        Ok(member) => member,
        Err(_) => return Ok(())
    };

    let footer = CreateEmbedFooter::new("Use it with .item <item> <@user> | Help with .help");
    let author = CreateEmbedAuthor::new("\u{200B}").icon_url(member.face());

    let inventory = compile_inventory(collection.clone(), msg.author.id).await;
    let skins = compile_skins(collection, msg.author.id).await;

    let embed = CreateEmbed::default().title(format!("{}'s Inventory", member.display_name()))
    .author(author)
    .field("\u{200B}", &inventory, true)
    .footer(footer)
    .color(Color::from_rgb(255, 134, 134));

    let builder = CreateMessage::new().add_embed(embed).button(next_page_button("Skins", "<a:alienfish:1153241407453143090>".parse().unwrap()));
    let message = msg.channel_id.send_message(&ctx.http, builder).await?;
    button_interaction(&ctx, message, &inventory, &skins, member).await;
    Ok(())
}

async fn button_interaction(ctx: &Context, msg: Message, inventory: &String, skins: &String, member: Member) {
    let button_text = ["Items", "Skins"];
    let mut button_index = 1;
    let mut interaction = msg.await_component_interaction(&ctx.shard)
    .timeout(Duration::from_secs(60)).stream();

    while let Some(interaction) = interaction.next().await {
        let author = CreateEmbedAuthor::new(format!("{}'s Inventory", member.display_name())).icon_url(member.face());

        if button_index == 0 {
            let footer = CreateEmbedFooter::new("[Items]").icon_url("https://cdn.discordapp.com/emojis/1141803332088905798.webp?size=96&quality=lossless");
            let embed = CreateEmbed::new().title(format!("{}'s Inventory", member.display_name()))
            .author(author)
            .field("Items", inventory, true)
            .footer(footer)
            .color(Color::from_rgb(255, 134, 134));
            interaction.create_response(&ctx,
            serenity::builder::CreateInteractionResponse::UpdateMessage(
                CreateInteractionResponseMessage::default().add_embed(embed)
                .button(next_page_button( button_text[button_index], "<:chest:1156928067948781620>".parse().unwrap()))
            )).await.unwrap();
            button_index = 1;
        } else {
            let footer = CreateEmbedFooter::new("[Skins]").icon_url("https://cdn.discordapp.com/emojis/1141803332088905798.webp?size=96&quality=lossless");
            let embed = CreateEmbed::new()
            .author(author)
            .field("Skins", skins, true)
            .footer(footer)
            .color(Color::from_rgb(255, 134, 134));
            interaction.create_response(&ctx,
            serenity::builder::CreateInteractionResponse::UpdateMessage(
                CreateInteractionResponseMessage::default().add_embed(embed)
                .button(next_page_button( button_text[button_index], "<a:alienfish:1153241407453143090>".parse().unwrap()))
            )).await.unwrap();
            button_index = 0;
        }
    }
}

fn next_page_button(name: &str, emoji: ReactionType) -> CreateButton {
    CreateButton::new("next")
        .label(name)
        .emoji(emoji)
}

async fn compile_inventory(collection: Collection<Document>, user_id: UserId) -> String {
    let filter = doc! {"user_id": user_id.get() as i64};
    let mut items = String::new();
    if let Some(document) = collection.find_one(filter.clone(), None).await.expect("I couldn't find that person, and I looked far and wide. **Source: trust me bro**") {
        if let Ok(cloaks) = document.get_i64("cloak") {
            if cloaks >= 1 as i64 {
                items.push_str(&format!("⤷ <:potion1:1141774817108971601> |  **{}x** (`Cloak`)\n", cloaks));
            }
        }
        if let Ok(uwuifiers) = document.get_i64("uwuify") {
            if uwuifiers >= 1 as i64 {
                items.push_str(&format!("⤷ <:UwUifier:1160202211826085949> |  **{}x** (`Uwuifier`)\n", uwuifiers));
            }
        }
    }
    if items.is_empty() {
        return "`It sure is empty in here.. like my soul`".to_string();
    }
    items
}

async fn compile_skins(collection: Collection<Document>, user_id: UserId) -> String {
    let mut all_skins = String::new();
    let filter = doc! {"user_id": user_id.get() as i64};
    if let Some(document) = collection.find_one(filter.clone(), None).await.expect("I couldn't find that person, and I looked far and wide. **Source: trust me bro**") {
        if let Ok(dinorace_skin) = document.get_str("dinorace_skin") {
            all_skins.push_str(&format!("Equipped: {}\n\n", dinorace_skin));
        }
        all_skins.push_str("In inventory: \n");
        if let Ok(skins) = document.get_array("owned_skins") {
            for skin in skins {
                all_skins.push_str(&format!("[{}],", skin.as_str().unwrap()));
            }
            return all_skins;
        }
    }
    "`It sure is empty in here.. like my soul`".to_string()
}