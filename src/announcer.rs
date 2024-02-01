use std::sync::Arc;

use chrono::Datelike;
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

    fn date_string(&self) -> String {
        if let Some(dt) = self.start_time() {
            dt.format("%A %d %b").to_string()
        } else {
            self.date.clone()
        }
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

    // TODO remove once satisfied with testing.
    events.push(Event {
        name: "Test Event 1".to_string(),
        date: "02 Feb 2024".to_string(),
        location: "Location 1".to_string(),
        category: None,
        attending: None,
        notes: Some("Notes for event".to_string()),
    });
    events.push(Event {
        name: "Test Event 2".to_string(),
        date: "03 Feb 2024".to_string(),
        location: "Location 2".to_string(),
        category: None,
        attending: None,
        notes: Some("Notes for event".to_string()),
    });

    Ok(events)
}

async fn send_embed(
    embed: discord::Embed,
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
        embeds: Some(vec![embed]),
        ..Default::default()
    };
    req::post(uri, body).await
}

async fn announce(channels: &[discord::Snowflake]) {
    const DATE_FORMAT: &str = "%A %d/%m";

    let mut events = match load_announcements().await {
        Ok(events) => events,
        Err(e) => {
            eprintln!("Failed to load events: {e}");
            return;
        }
    };

    let now = chrono::Local::now();

    // Filter for events in coming week.
    events.retain(|e| {
        if let Some(start) = e
            .start_time()
            .and_then(|t| t.signed_duration_since(now).to_std().ok())
        {
            start < std::time::Duration::from_secs(60 * 60 * 24 * 7 + 60 * 60 * 15)
        } else {
            false
        }
    });

    if events.is_empty() {
        return;
    }

    let desc = if let Some(end) = now.checked_add_days(chrono::Days::new(7)) {
        format!(
            "{} through {}",
            now.format(DATE_FORMAT),
            end.format(DATE_FORMAT)
        )
    } else {
        format!("Week beginning {}", now.format(DATE_FORMAT))
    };
    let mut embed = discord::Embed::new("Events this Week", &desc);

    for event in events {
        let date = event.date_string();
        let notes = if let Some(notes) = event.notes {
            format!(". {notes}")
        } else {
            String::new()
        };
        embed.add_field(event.name, format!("{}, {}{}", date, event.location, notes));
    }

    embed.add_field(String::new(), "@everyone".to_string());

    for channel in channels {
        if let Err(e) = send_embed(embed.clone(), channel).await {
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

fn next_sunday() -> Option<chrono::DateTime<chrono::Local>> {
    const DAY: u64 = 7; // 7 = sunday, 6 = saturday and so on.
    const HOUR: u32 = 9;
    const MIN: u32 = 0;

    let now = chrono::Local::now();
    let nd = now.date_naive();
    nd.checked_sub_days(chrono::Days::new(
        nd.weekday().num_days_from_sunday().into(),
    ))
    .and_then(|nd| nd.checked_add_days(chrono::Days::new(DAY)))
    .and_then(|nd| chrono::NaiveTime::from_hms_opt(HOUR, MIN, 0).map(|nt| nd.and_time(nt)))
    .and_then(|ndt| chrono::TimeZone::from_local_datetime(&chrono::Local, &ndt).single())
    .and_then(|dt| {
        if dt.signed_duration_since(now) < chrono::Duration::zero() {
            dt.checked_add_days(chrono::Days::new(7))
        } else {
            Some(dt)
        }
    })
}

pub async fn run_announcer(mut commands: UnboundedReceiver<AnnouncerCommand>) {
    let channels = Arc::new(Mutex::new(load_channels().await));

    // Handle commands to register and deregister channels for announcements.
    // The other end of this channel is used to pass commands through from
    // discord interactions.
    let command_channels = channels.clone();
    tokio::task::spawn(async move {
        while let Some(command) = commands.recv().await {
            if let Some(updated) = handle_command(command).await {
                let mut lock = command_channels.lock().await;
                *lock = updated;
            }
        }
    });

    // Publish announcements to all registered channels every sunday morning.
    tokio::task::spawn(async move {
        loop {
            // Wait until 9:00 next sunday morning or 1 week on error.
            let until_next_sunday = if let Some(dt) = next_sunday() {
                println!("Sleeping until: {dt} for next announcement.");
                dt.signed_duration_since(chrono::Local::now()).to_std().ok()
            } else {
                None
            };

            let duration = until_next_sunday
                .unwrap_or_else(|| std::time::Duration::from_secs(60 * 60 * 24 * 7));
            let instant = std::time::Instant::now() + duration;

            tokio::time::interval_at(
                tokio::time::Instant::from_std(instant),
                std::time::Duration::from_secs(1),
            )
            .tick()
            .await;

            // Run announcement.
            announce(&channels.lock().await).await;
        }
    });
}
