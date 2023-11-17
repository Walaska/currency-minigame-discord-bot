mod currency_commands;
use std::env;
use std::sync::Arc;
use mongodb::options::ClientOptions;
use serenity::all::Message;
use serenity::async_trait;
use serenity::framework::standard::macros::group;
use serenity::framework::StandardFramework;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use serenity::gateway::ShardManager;
use tracing::error;

use crate::currency_commands::daily::*;
use crate::currency_commands::slots::*;
use crate::currency_commands::give::*;
use crate::currency_commands::coins::*;
use crate::currency_commands::flip::*;
use crate::currency_commands::shop::*;
use crate::currency_commands::buy::*;
use crate::currency_commands::item::*;
use crate::currency_commands::inv::*;
use crate::currency_commands::help::*;
use crate::currency_commands::dinorace::*;
pub struct ShardManagerContainer;

struct MongoDb;

struct Handler;

impl TypeMapKey for MongoDb {
    type Value = Arc<mongodb::Client>;
}

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

#[group]
#[commands(daily, slots, give, coins, flip, shop, buy, item, inv, help, dinorace)]
struct General;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
    async fn message(&self, ctx: Context, msg: Message) {
        currency_commands::shop_items::uwuify::message(&ctx, &msg).await;
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().expect("Failed to load .env file");
    tracing_subscriber::fmt::init();
    let token = env::var("DISCORD_TOKEN").expect("No bot token!");
    let framework = StandardFramework::new()
    .bucket("slots", |b| b.delay(3).limit(1)).await
    .group(&GENERAL_GROUP);
    framework.configure(|c| c.prefix("."));

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_MEMBERS;
    let mut client = Client::builder(&token, intents)
        .framework(framework)
        .event_handler(Handler)
        .await
        .expect("Error creating the client! :(");


    {
        let mut data = client.data.write().await;
        if let Ok(client_option) = ClientOptions::parse(format!("mongodb+srv://default:{}@digitalart.k4xqkao.mongodb.net/?retryWrites=true&w=majority", env::var("MONGO_PASSWORD").expect("No mongo password!"))).await {
            let client = mongodb::Client::with_options(client_option).unwrap();
            data.insert::<MongoDb>(Arc::new(client));
        }
    }
    
    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    };
}
