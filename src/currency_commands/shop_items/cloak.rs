use tokio::time::Duration;
use serenity::{
    model::prelude::*,
    prelude::*,
    builder::EditMember,
};

const CLOAK_ROLE_ID: u64 = 1137803273164836884;  // Changed this from Option<NonZeroU64> to just u64

pub async fn cloak(ctx: &Context, guild_id: GuildId, user_id: UserId) {
    let old_roles = match guild_id.member(ctx, user_id).await {
        Ok(member) => member.roles,
        Err(e) => {
            println!("{:?}", e);
            return;
        }
    };

    let empty_roles: &[RoleId] = &[];

    let display_name = match ctx.http.get_member(guild_id, user_id).await {
        Ok(member) => member.display_name().to_string(),
        Err(_) => "Richard_James".to_string(),
    };

    let builder = EditMember::new()
        .roles(empty_roles)
        .nickname("឵឵");

    if let Err(e) = guild_id.edit_member(ctx, user_id, builder.clone()).await {
        println!("{:?}", e);
    }

    let cloak_role_id = RoleId::from(CLOAK_ROLE_ID);  // Using the 'from' trait to create a RoleId

    if let Err(e) = ctx.clone().http.add_member_role(guild_id, user_id, cloak_role_id, Some("Cloak role given")).await {
        println!("-------------> {:?}", e);
    }

    tokio::time::sleep(Duration::from_secs(180)).await;

    if let Err(e) = guild_id.edit_member(ctx, user_id, builder.clone()).await {
        println!("{:?}", e);
    }

    let builder = EditMember::new()
        .nickname(display_name);

    if let Err(e) = guild_id.edit_member(ctx, user_id, builder).await {
        println!("{:?}", e);
    }

    for role_id in old_roles {
        if let Err(e) = ctx.clone().http.add_member_role(guild_id, user_id, role_id, Some("Old role given")).await {
            println!("Error adding old role: {:?}", e);
        }
    }
}