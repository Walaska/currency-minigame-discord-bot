use std::time::Duration;
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::*,
    prelude::*, builder::{CreateEmbed, CreateMessage, CreateButton, CreateInteractionResponseMessage, CreateEmbedFooter, CreateEmbedAuthor}, futures::StreamExt,
};

#[command]
async fn shop(ctx: &Context, msg: &Message, mut _args: Args) -> CommandResult {
    let builder = CreateMessage::new().add_embed(get_items_embed()).button(next_page_button("Skins", "<:chest:1156928067948781620>".parse().unwrap()));
    let message = msg.channel_id.send_message(&ctx.http, builder).await?;
    button_interaction(&ctx, message).await;
    Ok(())
}

async fn button_interaction(ctx: &Context, msg: Message) {
    let button_text = ["Items", "Skins"];
    let mut button_index = 1;
    let mut interaction = msg.await_component_interaction(&ctx.shard)
    .timeout(Duration::from_secs(60)).stream();

    while let Some(interaction) = interaction.next().await {

        if button_index == 0 {
            interaction.create_response(&ctx,
            serenity::builder::CreateInteractionResponse::UpdateMessage(
                CreateInteractionResponseMessage::default().add_embed(get_items_embed())
                .button(next_page_button( button_text[button_index], "<a:alienfish:1153241407453143090>".parse().unwrap()))
            )).await.unwrap();
            button_index = 1;
        } else {
            interaction.create_response(&ctx,
            serenity::builder::CreateInteractionResponse::UpdateMessage(
                CreateInteractionResponseMessage::default().add_embed(get_skins_embed())
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

fn get_skins_embed() -> CreateEmbed {
    let footer = CreateEmbedFooter::new("To buy a skin, use .buy #");
    let author = CreateEmbedAuthor::new("[Shop] - Skins").icon_url("https://cdn.discordapp.com/emojis/1109794189341904958.webp?size=96&quality=lossless");

    let embed = CreateEmbed::default()
        .footer(footer)
        .author(author)
        .field("**Common**", "<a:DAcoin2:1137457024729370754> [__`100`__]\n[**2**] <a:steve:1153241655944675418> Steve\n[**3**] <a:riley:1153241629931610132> Riley\n", true)
        .field("**Uncommon**", "<a:DAcoin2:1137457024729370754> [__`1k`__]\n[**4**] <a:angryR:1153241416017924116> Angry Riley\n", true)
        .field("**Rare**", "<a:DAcoin2:1137457024729370754> [__`5k`__]\n[**5**] <a:Choctrex:1153241426688229406> Choco Trex\n[**6**] <a:duriansteve:1153241508854648863> Durian Steve\n[**7**] <a:fish:1153241519231356978> Cat Fish\n", true)
        .field("**Epic**", "<a:DAcoin2:1137457024729370754> [__`25k`__]\n[**8**] <a:Rileywiz:1153241649900687420> Riley Potter\n[**9**] <a:maintanance:1153241526562996314> Trex Builder\n[**10**] <a:turtlesteve:1153241671648170035> Turtle Steve\n[**11**] <a:alienfish:1153241407453143090> Alien Fish\n[**12**] <a:bags:1157977401616248842> Bags\n", true)
        .field("**Legendary**", "<a:DAcoin2:1137457024729370754> [__`100k`__]\n[**13**] <a:coolfish:1153241497278369812> Cool Fish\n[**14**] <a:socket:1157977422340296748> Socket", true)
        .field("**Mythic**", "<a:DAcoin2:1137457024729370754> [__`1m`__]\n[**15**] <:NoFreeArt:1002717330838663239> Free Art\n[**16**] <a:king:1157977404015378432> Kings", true)
        .color(Color::from_rgb(255, 134, 134));
    embed
}

fn get_items_embed() -> CreateEmbed {
    let footer = CreateEmbedFooter::new("Use it with .item <item> <@user> | Help with .help");
    let author = CreateEmbedAuthor::new("[Shop] - Items").icon_url("https://cdn.discordapp.com/emojis/1109794189341904958.webp?size=96&quality=lossless");

    let embed = CreateEmbed::default()
        .footer(footer)
        .author(author)
        .field("\u{200B}", "<a:spooky:1136720506805043341> `[cloak]` - **250** <a:DAcoin2:1137457024729370754>\n<:UwUifier:1160202211826085949> `[uwuify]` - **500** <a:DAcoin2:1137457024729370754>", false)
        .field("ᴄᴏᴍɪɴɢ sᴏᴏɴ™️", "- <:POHLICE:1137743401261998110> Jail\n- <a:fire:1044851524326653994> Streak save (subject to change)", false)
        .color(Color::from_rgb(255, 134, 134));
    embed
}