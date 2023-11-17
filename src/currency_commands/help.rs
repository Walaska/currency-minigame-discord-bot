use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::*,
    prelude::*, builder::{CreateEmbed, CreateMessage},
};

#[command]
async fn help(ctx: &Context, msg: &Message, mut _args: Args) -> CommandResult {

    let embed = CreateEmbed::default().title("Currency Help Menu")
        .description(format!("## Economy\n💳 `.buy <item name/number>` | Use this command to buy items from the shop\n<a:DAcoin2:1137457024729370754> `.coins` | Check your/other's current balance!\n📅 `.daily` | Claim your daily coins reward
                            \n## Games\n<a:EGHdinosaurjump:1156677975161454772> `.dinorace <bet> <speed>` | Race with FRENS. Who's dino is fastest? Win big, or lose big. Up 2 u.\n<a:flip:1137440717715808327> `.flip <head/tails> <bet amount>` | Take a chance and flip a coin to win more coins\n<a:slot:1157270569763475557> `.slots <bet amount>` | Try your luck with the slot machine and win some coins
                            \n## General\n<:inventory:1157270769248768078> `.inv` | View your inventory and see what items you own\n<:cart:1157270915919401021> `.shop` | Display the items available for purchase\n<a:gift:1157271158522134569> `.give <@user>` | Gift some of your coins to another user"))
    .color(Color::from_rgb(213, 237, 249));

    let builder = CreateMessage::new().add_embed(embed);

    msg.channel_id.send_message(&ctx.http, builder).await?;
    Ok(())
}
