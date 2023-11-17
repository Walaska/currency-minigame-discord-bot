use std::time::Duration;
use chrono::Utc;
use tokio::time::sleep;
use rand::{rngs::{OsRng, StdRng}, SeedableRng, Rng, RngCore};
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::*,
    prelude::*, builder::{CreateEmbed, CreateMessage, EditMessage, CreateButton, CreateSelectMenu, CreateSelectMenuOption, CreateInteractionResponseMessage, CreateActionRow}, futures::StreamExt,
};
use mongodb::{bson::{doc, Document}, Collection};
use crate::MongoDb;

const TRACK_SIZE: usize = 9;
const HORSE_MOVE_PROPABILITY: [f64; 3] = [0.3, 0.5, 0.7];
const TRACK_NUMBER_EMOJIS: [&str; 8] = ["<:1:1156668971781730385>", "<:2:1156668993243992174>",
                                        "<:3:1156669011875082332>", "<:4:1156669034151030977>",
                                        "<:5:1156669055671992340>", "<:6:1156669074672189521>",
                                        "<:7:1156669091235495966>", "<:8:1156669108335677501>"];
const TRACK_STRING: &str = "＿";

#[command]
async fn dinorace(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let mut bets = 0;

    let arg1 = args.single::<String>()?;
    let arg1_is_numeric = arg1.parse::<u64>().is_ok();
    let arg1_valid = arg1_is_numeric || arg1 == "fast" || arg1 == "slow" || arg1 == "normal";

    let arg2 = args.single::<String>()?;
    let arg2_is_numeric = arg2.parse::<u64>().is_ok();
    let arg2_valid = arg2_is_numeric || arg2 == "fast" || arg2 == "slow" || arg2 == "normal";

    if !arg1_valid || !arg2_valid {
        msg.reply(&ctx.http, "Oh noo, that's not right. Please specify the speed, you can choose between 'fast,' 'normal,' or 'slow' as your flavor! <a:SpeedL:1156676531251335188><a:SpeedR:1156676549572038726>").await?;
        return Err("Oh noo, that's not right. Please specify the speed, you can choose between 'fast,' 'normal,' or 'slow' as your flavor! <a:SpeedL:1156676531251335188><a:SpeedR:1156676549572038726>".into());
    }

    let mut speed = 1;
    if arg1 == "fast" || arg2 == "fast" {
        speed = 2;
    } else if arg1 == "slow" || arg2 == "slow" {
        speed = 0;
    }
    
    if arg1_is_numeric {
        bets = arg1.parse().unwrap();
    } else {
        if arg2 == "fast" {
            speed = 2;
        } else if arg2 == "slow" {
            speed = 0;
        }
    }

    if arg2_is_numeric {
        bets = arg2.parse().unwrap();
    } else {
        if arg1 == "fast" {
            speed = 0;
        } else if arg1 == "slow" {
            speed = 2;
        }
    }

    let mut tracks = vec![];
    let timestamp = Utc::now().timestamp() + 29;

    let button = join_game_button(&format!("Join Race [{}]", bets), "<a:woow:1021061003649220629>".parse().unwrap());
    let embed = CreateEmbed::default().title("Dino race is about to start! 🦕")
    .description("You don't wanna race? Yes u do. No excuses. **Bring ur racesaurus!**\n*Max players:* **8**")
    .color(Color::from_rgb(255, 134, 134));
    let builder = CreateMessage::new().add_embed(embed).button(button);
    let mut message = msg.channel_id.send_message(&ctx.http, builder).await?;
    let players = lobby(&ctx, &msg, &mut tracks, &mut message, timestamp, bets).await.expect("Error getting players");
    if players.len() > 1 {
        for player in &players {
            take_coins(player.user.id, &ctx, bets).await;
        }

        let builder = EditMessage::new().button(join_game_button("Race Begun", "<a:Party_Dino:1156670139874410587>".parse().unwrap()).style(ButtonStyle::Success).disabled(true));
        message.edit(&ctx.http, builder).await?;
        let win_amount = bets * players.len() as u64;
        let winners = start_race(&mut tracks, &mut message, &ctx, &players, speed).await.expect("Error in race loop");
        if winners.len() == 1 {
            let win_msg = format!("_*{}* on track *{}* won!_ Absolutely built different.\nThey nabbed **{}** <a:DAcoin:1137440625017503855> coins. <:sparkleeyes:1087482184929136680>
", players[winners[0]].display_name(), winners[0] as i32 + 1, win_amount);
            handle_winner(players[winners[0]].user.id, &ctx, win_amount as i64).await;
            _ = result(ctx, &mut tracks, &players, &mut message, win_msg, winners[0]).await;
        } else if winners.len() > 1 {
            let winner = handle_tie(winners);
            let win_msg = format!("_Woa. A TIE? <:dice:1156682671418769480> Guess we're rolling the dice on who wins. Guess it's *{}* on track *{}*.The **{}** <a:DAcoin:1137440625017503855> are yours._", players[winner].display_name(), winner as i32 + 1, win_amount);
            handle_winner(players[winner].user.id, &ctx, win_amount as i64).await;
            _ = result(ctx, &mut tracks, &players, &mut message, win_msg, winner).await;
        }
    } else {
        _ = not_enough_players(ctx, &mut message).await;
    }

    println!("Race Finished");
    Ok(())
}

async fn not_enough_players(ctx: &Context, message: &mut Message) -> Result<(), Box<dyn std::error::Error>> {
    let builder = EditMessage::new().add_embed(CreateEmbed::default()
    .title("~~Dino Race~~ <a:EGHdinosaurjump:1156677975161454772>")
    .description("Didn't get enough players to start the race. T_T\nYou need at least one other friend.")
    .color(Color::from_rgb(255, 134, 134)))
    .button(join_game_button("Not Enough Players", "<:regretpainandsufferingwhyrwehere:1143176704706228284>".parse().unwrap()).disabled(true));
    message.edit(&ctx.http, builder).await?;
    Ok(())
}

async fn result(ctx: &Context, tracks: &mut Vec<Vec<String>>, players: &Vec<Member>, message: &mut Message, win_msg: String, winner: usize) -> Result<(), Box<dyn std::error::Error>> {
    let builder = EditMessage::new().add_embed(CreateEmbed::default()
    .title("Dino Race over! <a:EGHdinosaurjump:1156677975161454772>")
    .field("Tracks", get_track_string(&tracks), true)
    .field("Players", get_players(&players, winner), true)
    .field("Winner", win_msg, false)
    .color(Color::from_rgb(255, 134, 134)));
    message.edit(&ctx.http, builder).await?;
    Ok(())
}

async fn lobby(ctx: &Context, msg: &Message, tracks: &mut Vec<Vec<String>>, message: &mut Message, timestamp: i64, bets: u64) -> Result<Vec<Member>, Box<dyn std::error::Error>> {

    let mut players = vec![];

    let mut interaction = message.await_component_interaction(&ctx.shard)
        .timeout(Duration::from_secs(30)).stream();

    while let Some(interaction) = interaction.next().await {

        if check_players(&interaction.clone().member.unwrap(), &players) {
            interaction.create_response(&ctx,
                serenity::builder::CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::default().content("*In the vast cosmic theater, where the interplay of fate and free will compose the grand symphony of existence, your being has transcended mere participation. You have been cast not merely as a spectator but as an integral performer, your every step choreographed by the intricate dance of life's complexities. The race you find yourself in is but a fleeting echo of a greater journey, one that began with the spark of consciousness and will continue to weave through the tapestry of time and space, until the last notes of your unique melody merge once again with the eternal silence of the universe.*\n\n**(You're already in the race, btw)**").ephemeral(true)
                )).await?;
        } else if !check_coins(interaction.clone().member.unwrap().user.id, &ctx, bets).await {
            interaction.create_response(&ctx,
                serenity::builder::CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::default().content("*Oh noes, looks like you don't have enough coinsies to join our little race, such a shame uwu 🌸✨*").ephemeral(true)
                )).await?;
        } else {
            players.push(interaction.clone().member.unwrap());
            tracks.push(compile_track(&ctx, &interaction.clone().member.unwrap()).await);
    
            interaction.create_response(&ctx,
                serenity::builder::CreateInteractionResponse::UpdateMessage(
                    CreateInteractionResponseMessage::default().add_embed(compile_lobby_embed(&players, &tracks, timestamp, bets))
                )).await?;
            if players.len() == 4 {
                return Ok(vec![players[0].clone(), players[1].clone(), players[2].clone(), players[3].clone()]);
            }
        }
    }

    if players.len() < 2 {
        return Ok(vec!())
    }

    Ok(players)
}

async fn handle_winner(user_id: UserId, ctx: &Context, win_amount: i64) {
    let client = {
        let data_read = ctx.data.read().await;
        data_read.get::<MongoDb>().expect("Expected MongoDb").clone()
    };
    let db = client.database("currency");
    let collection = db.collection::<Document>("user_currency");
    let filter = doc! {"user_id": user_id.get() as i64};
    let options = mongodb::options::UpdateOptions::builder().upsert(true).build();
    let update_doc = doc! {"$inc": {"currency": win_amount}};
    collection.update_one(filter.clone(), update_doc, options).await.unwrap();
}

fn check_players(member: &Member, players: &Vec<Member>) -> bool {
    players.iter().any(|player| player.user.id == member.user.id)
}

async fn check_coins(user_id: UserId, ctx: &Context, bet_amount: u64) -> bool {
    let client = {
        let data_read = ctx.data.read().await;
        data_read.get::<MongoDb>().expect("Expected MongoDb").clone()
    };
    let db = client.database("currency");
    let collection = db.collection::<Document>("user_currency");
    let filter = doc! {"user_id": user_id.get() as i64};
    if let Some(document) = collection.find_one(filter, None).await.expect("User not found") {
        if let Ok(currency) = document.get_i64("currency") {
            if currency >= bet_amount as i64 && bet_amount > 0 {
                return true;
            }
        }
    }
    false
}

async fn take_coins(user_id: UserId, ctx: &Context, bet_amount: u64) {
    let client = {
        let data_read = ctx.data.read().await;
        data_read.get::<MongoDb>().expect("Expected MongoDb").clone()
    };
    let db = client.database("currency");
    let collection = db.collection::<Document>("user_currency");
    let filter = doc! {"user_id": user_id.get() as i64};
    let options = mongodb::options::UpdateOptions::builder().upsert(true).build();
    let update_doc = doc! {"$inc": {"currency": (bet_amount as i64 * -1)}};
    collection.update_one(filter.clone(), update_doc, options).await.unwrap();
}

fn compile_lobby_embed(players: &Vec<Member>, tracks: &Vec<Vec<String>>, timestamp: i64, bet: u64) -> CreateEmbed {
    let embed = CreateEmbed::default().title("Dino race is about to start! 🦕")
    .description(format!("## It starts **<t:{}:R>**!!\n**PRIZE POOL:** **{}** <a:DAcoin:1137440625017503855> coins", timestamp, bet * players.len() as u64))
    .field("Tracks", get_track_string(&tracks), true)
    .field("Players", get_players(&players, 100), true)
    .color(Color::from_rgb(255, 134, 134));
    embed
}

async fn start_race(tracks: &mut Vec<Vec<String>>, message: &mut Message, ctx: &Context, players: &Vec<Member>, speed: u64) -> Result<Vec<usize>, Box<dyn std::error::Error>> {
    loop {
        sleep(Duration::from_millis(500)).await;
        for track in &mut* tracks {
            if check_if_move(speed) {
                move_horse(track);
            }
        }

        let builder = EditMessage::new().add_embed(CreateEmbed::default()
        .title("Dino's RACIN rn! 🦕")
        .field("Tracks", get_track_string(&tracks), true)
        .field("Players", get_players(&players, 100), true)
        .color(Color::from_rgb(255, 134, 134)));
        message.edit(&ctx.http, builder).await?;
        let winners = check_for_winners(&tracks);
        if winners.len() > 0 {
            return Ok(winners);
        }
    }
}

fn get_players(players: &Vec<Member>, winner: usize) -> String {
    let mut track_string = String::new();
    let mut index = 0;
    for player in players {
        track_string.push_str("`");
        track_string.push_str(player.display_name());
        if index == winner {
            track_string.push_str(" 👑");
        }
        track_string.push_str("`\n");
        index += 1;
    }
    track_string
}

fn get_track_string(tracks: &Vec<Vec<String>>) -> String {
    let mut track_string = String::new();
    let mut index = 0;
    for track in tracks {
        track_string.push_str(&format!("{} :checkered_flag:", TRACK_NUMBER_EMOJIS[index]));
        track_string.push_str(&track.join(""));
        track_string.push_str("\n");
        index += 1;
    }
    track_string
}

fn check_for_winners(tracks: &Vec<Vec<String>>) -> Vec<usize> {
    let mut winners = vec![];
    for (index, track) in tracks.iter().enumerate() {
        if track[0] != TRACK_STRING {
            winners.push(index);
        }
    }
    winners
}

fn handle_tie(winners: Vec<usize>) -> usize {
    let seed = [42u8; 32];
    let mut rng = StdRng::from_seed(seed);
    let index = rng.gen_range(0..winners.len());
    winners[index]
}

fn move_horse(track: &mut Vec<String>) {
    let horse_index = track.iter().position(|s| s != TRACK_STRING).unwrap();
    let skin = track[horse_index].clone();
    if horse_index > 0 {
        track[horse_index] = TRACK_STRING.to_string();
        track[horse_index - 1] = skin;
    }
}

fn check_if_move(speed: u64) -> bool {
    let mut seed_bytes = [0u8; 32];
    let mut os_rng = OsRng::default();
    os_rng.fill_bytes(&mut seed_bytes);

    let mut rng = rand::rngs::StdRng::from_seed(seed_bytes);
    rng.gen::<f64>() < HORSE_MOVE_PROPABILITY[speed as usize]
}

async fn check_player_skin(ctx: &Context, player: &Member) -> String {
    let client = {
        let data_read = ctx.data.read().await;
        data_read.get::<MongoDb>().expect("Expected MongoDb").clone()
    };
    let db = client.database("currency");
    let collection = db.collection::<Document>("user_inventory");
    let filter = doc! {"user_id": player.user.id.get() as i64};
    if let Some(document) = collection.find_one(filter, None).await.expect("User not found") {
        if let Ok(skin) = document.get_str("dinorace_skin") {
            return skin.to_string();
        }
    }
    "<a:trexxx:1153241661984485426>".to_string()
}

async fn compile_track(ctx: &Context, player: &Member) -> Vec<String> {
    let mut track = vec![];
    for _ in 0..TRACK_SIZE {
        track.push(TRACK_STRING.to_string());
    }
    track.push(check_player_skin(ctx, player).await);
    track
}

fn join_game_button(name: &str, emoji: ReactionType) -> CreateButton {
    CreateButton::new("join")
        .label(name)
        .emoji(emoji)
}