#![deny(clippy::all)]
#![deny(clippy::pedantic)]

use std::{env, time::Duration};
use serenity::all::{Attachment, AttachmentType};
use serenity::builder::{CreateEmbed, ExecuteWebhook, CreateAttachment};
use url::Url;
use serde_json::json;
use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready},
    prelude::*,
};
use mongodb::{bson::{doc, Document,  DateTime}, Collection, Database};
use crate::MongoDb;

pub async fn message(ctx: &Context, msg: &Message) {
        if !check_for_uwuify(&ctx, &msg).await { return; }
        if is_valid_url(&msg.content) { return; }
        if msg.webhook_id.is_none() {
            let mut webhooks = match ctx
                .http
                .get_channel_webhooks(msg.channel_id)
                .await
            {
                Err(why) => return eprintln!("Error getting webhooks: {:?}", why),
                Ok(list) => list,
            };
            let webhook = match if webhooks.is_empty() {
                match ctx
                    .http
                    .create_webhook(msg.channel_id, &json!({"name": "UwU wats dis"}), Some("Uwuify webhook creation"))
                    .await
                {
                    Err(why) => {
                        eprintln!("Error creating webhook: {:?}", why);
                        None
                    }
                    Ok(webhook) => Some(webhook),
                }
            } else {
                webhooks.pop()
            } {
                Some(webhook) => webhook,
                None => return,
            };
            let delete_msg = msg.clone();
            let nick = msg
                .author_nick(&ctx)
                .await
                .unwrap_or_else(|| msg.author.name.clone());
            if let Err(why) = webhook
                .execute(&ctx, true, webhook_builder(msg.author.avatar_url().unwrap_or_default(), uwuifier::uwuify_str_sse(&*msg.content), nick, attachments_to_create_attachments(&ctx, msg.attachments.clone()).await))
                .await
            {
                return eprintln!("Error executing webhook: {:?}", why);
            };
            if let Err(why) = delete_msg.delete(&ctx).await {
                return eprintln!("Error deleting message: {:?}", why);
            };
        }
    }

async fn check_for_uwuify(ctx: &Context, msg: &Message) -> bool {
    let client = {
        let data_read = ctx.data.read().await;
        data_read.get::<MongoDb>().expect("Expected MongoDb").clone()
    };
    let db = client.database("currency");
    let collection = db.collection::<Document>("user_modifiers");
    let filter = doc! {"user_id": msg.author.id.get() as i64};
    if let Some(document) = collection.find_one(filter, None).await.expect("User not found") {
        if let Ok(uwuify) = document.get_bool("uwuify") {
            return uwuify;
        }
    }
    false
}

async fn attachments_to_create_attachments(ctx: &Context, attachments: impl IntoIterator<Item = Attachment>) -> Vec<CreateAttachment> {
    let mut create_attachments: Vec<CreateAttachment> = vec![];
    for attachment in attachments {
        create_attachments.push(CreateAttachment::url(&ctx.http, &attachment.url).await.expect("Attachment not found"));
    }
    create_attachments
}

fn is_valid_url(url_str: &str) -> bool {
    match Url::parse(url_str) {
        Ok(_) => true,
        Err(_) => false,
    }
}

fn webhook_builder(avatar_url: String, content: String, nick: String, files: Vec<CreateAttachment>) -> ExecuteWebhook {
    let mut builder = ExecuteWebhook::new()
    .avatar_url(avatar_url)
    .content(content)
    .username(nick)
    .files(files);
    builder
}