use serde::Serialize;
use tokio::sync::mpsc::UnboundedReceiver;

use crate::{discord, req, Result};

pub enum AnnouncerCommand {
    Announce(discord::Snowflake),
    RegisterChannel(discord::Snowflake),
    UnregisterChannel(discord::Snowflake),
}

async fn send_announcement(
    content: String,
    channel: &discord::Snowflake,
) -> Result<discord::Message> {
    #[derive(Default, Serialize)]
    struct CreateMessageRequest {
        content: Option<String>,
        nonce: Option<String>,
        tts: Option<bool>,
        embeds: Option<Vec<discord::Embed>>,
        allowed_mentions: Option<discord::AllowedMentions>,
        message_reference: Option<discord::MessageReference>,
        components: Option<Vec<discord::MessageComponent>>,
        sticker_ids: Option<Vec<discord::Snowflake>>,
        payload_json: Option<String>,
        attachments: Option<Vec<discord::Attachment>>,
        flags: Option<i32>,
    }

    let uri = req::api_uri(format!("/channels/{channel}/messages"));
    let body = CreateMessageRequest {
        content: Some(content),
        ..Default::default()
    };
    req::post(uri, body).await
}

pub async fn run_announcer(commands: UnboundedReceiver<AnnouncerCommand>) {}
