use std::sync::Arc;

use serde::Serialize;
use tokio::sync::{mpsc::UnboundedReceiver, Mutex};

use crate::{
    consts::EVENTS_SHEET_CSV,
    csv::{load_csv, parse_csv, write_csv},
    discord, req, Result,
};

pub enum AnnouncerCommand {
    RegisterChannel(discord::Snowflake),
    UnregisterChannel(discord::Snowflake),
}

const CHANNELS_CSV: &str = "channels.csv";

struct Event {
    name: String,
    date: String,
    location: String,
    category: Option<String>,
    attending: Option<String>,
    notes: Option<String>,
}

impl Event {
    fn start_time(&self) -> Option<chrono::DateTime<chrono::Local>> {
        let parts: Vec<&str> = self.date.split(' ').collect();
        let day = parts.first();
        let month = parts.get(1);
        let year = parts.get(2);

        let (Some(d), Some(m), Some(y)) = (day, month, year) else {
            return None;
        };

        let day: u32 = if d.contains('-') {
            if let Some((from, _to)) = d.split_once('-') {
                from.parse().ok()?
            } else {
                return None;
            }
        } else {
            d.parse().ok()?
        };

        let month: chrono::Month = m.parse().ok()?;
        let year: i32 = y.parse().ok()?;

        let naive = chrono::NaiveDate::from_ymd_opt(year, month.number_from_month(), day)?;
        let naive = naive.and_time(chrono::NaiveTime::MIN);
        let date = chrono::TimeZone::from_local_datetime(&chrono::Local, &naive);
        date.single()
    }
}

async fn load_announcements() -> Result<Vec<Event>> {
    let csv = req::get(EVENTS_SHEET_CSV).await?;
    let data = parse_csv(&csv)?;
    let mut events = Vec::new();
    for row in data.into_iter().skip(2) {
        let name = row.first();
        let date = row.get(1);
        let location = row.get(2);
        let category = row.get(3);
        let attending = row.get(4);
        let notes = row.get(5);

        if let (Some(name), Some(date), Some(location)) = (name, date, location) {
            events.push(Event {
                name: name.clone(),
                date: date.clone(),
                location: location.clone(),
                category: category.cloned(),
                attending: attending.cloned(),
                notes: notes.cloned(),
            })
        }
    }

    Ok(events)
}

async fn send_message(content: String, channel: &discord::Snowflake) -> Result<discord::Message> {
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

async fn announce(channels: &[discord::Snowflake]) {
    let Ok(events) = load_announcements().await else {
        return;
    };

    let mut message = String::new();
    for event in events {
        message.push_str(&event.name);
        message.push('|');
        message.push_str(&format!("{:?}", event.start_time()));
        message.push('\n');
    }

    for channel in channels {
        if let Err(e) = send_message(message.clone(), channel).await {
            eprintln!("{e}");
        }
    }
}

async fn load_channels() -> Vec<discord::Snowflake> {
    let csv = if let Ok(csv) = load_csv(CHANNELS_CSV).await {
        csv
    } else {
        Vec::new()
    };

    csv.into_iter()
        .filter_map(|row| row.into_iter().next())
        .collect()
}

async fn save_channels(channels: &[discord::Snowflake]) {
    if let Err(e) = write_csv(
        &channels.iter().map(|id| vec![id.clone()]).collect(),
        CHANNELS_CSV,
    )
    .await
    {
        eprintln!("{e}");
    }
}

async fn register_channel(channel: discord::Snowflake) -> Result<Vec<discord::Snowflake>> {
    let mut registered = load_channels().await;

    if !registered.contains(&channel) {
        registered.push(channel);
        save_channels(&registered).await;
    }

    Ok(registered)
}

async fn unregister_channel(channel: &discord::Snowflake) -> Result<Vec<discord::Snowflake>> {
    let mut registered = load_channels().await;

    if registered.contains(channel) {
        registered.retain(|id| id != channel);
        save_channels(&registered).await;
    }

    Ok(registered)
}

async fn handle_command(command: AnnouncerCommand) -> Option<Vec<discord::Snowflake>> {
    match command {
        AnnouncerCommand::RegisterChannel(id) => {
            if let Ok(registered) = register_channel(id).await {
                Some(registered)
            } else {
                None
            }
        }
        AnnouncerCommand::UnregisterChannel(id) => {
            if let Ok(registered) = unregister_channel(&id).await {
                Some(registered)
            } else {
                None
            }
        }
    }
}

pub async fn run_announcer(mut commands: UnboundedReceiver<AnnouncerCommand>) {
    let channels = Arc::new(Mutex::new(load_channels().await));

    let command_channels = channels.clone();
    tokio::task::spawn(async move {
        while let Some(command) = commands.recv().await {
            if let Some(updated) = handle_command(command).await {
                let mut lock = command_channels.lock().await;
                *lock = updated;
            }
        }
    });

    // TODO sunday mornings instead of 30 seconds.
    tokio::task::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
        loop {
            announce(&channels.lock().await).await;
            interval.tick().await;
        }
    });
}
