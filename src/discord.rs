use std::collections::HashMap;

use serde::{Deserialize, Serialize};

type Snowflake = String;

#[derive(Debug, Deserialize)]
struct Channel {}

#[derive(Debug, Deserialize)]
struct User {}

#[derive(Debug, Deserialize)]
struct Member {}

#[derive(Debug, Deserialize)]
struct Message {}

#[derive(Debug, Deserialize)]
struct Entitlement {}

#[derive(Debug, Deserialize)]
struct ResolvedData {}

#[derive(Debug, Deserialize)]
enum ChannelType {}

#[derive(Debug, Deserialize)]
struct ApplicationCommandOptionChoice {}

#[derive(Debug)]
enum ApplicationCommandOptionType {
    SubCommand,
    SubCommandGroup,
    String,
    Integer,
    Boolean,
    User,
    Channel,
    Role,
    Mentionable,
    Number,
    Attachment,
}

impl ApplicationCommandOptionType {
    fn ordinal(&self) -> i32 {
        match self {
            Self::SubCommand => 1,
            Self::SubCommandGroup => 2,
            Self::String => 3,
            Self::Integer => 4,
            Self::Boolean => 5,
            Self::User => 6,
            Self::Channel => 7,
            Self::Role => 8,
            Self::Mentionable => 9,
            Self::Number => 10,
            Self::Attachment => 11,
        }
    }
}

#[derive(Debug, Deserialize)]
struct Localisations(HashMap<String, String>);

#[derive(Debug, Deserialize)]
pub struct ApplicationCommandOption {
    #[serde(rename = "type")]
    _type: i32,

    name: String,
    name_localizations: Option<Localisations>,
    description: String,
    description_localizations: Option<Localisations>,
    required: Option<bool>,
    choices: Option<Vec<ApplicationCommandOptionChoice>>,
    options: Option<Vec<ApplicationCommandOption>>,
    channel_types: Option<Vec<ChannelType>>,
    min_value: Option<f64>,
    max_value: Option<f64>,
    min_length: Option<i32>,
    max_length: Option<i32>,
    autocomplete: Option<bool>,
}

pub enum ApplicationCommandType {
    ChatInput,
    User,
    Message,
}

impl ApplicationCommandType {
    pub fn ordinal(&self) -> i32 {
        match self {
            Self::ChatInput => 1,
            Self::User => 2,
            Self::Message => 3,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ApplicationCommand {
    #[serde(rename = "type")]
    _type: Option<i32>,

    id: Snowflake,
    application_id: Snowflake,
    guild_id: Option<Snowflake>,
    name: String,
    name_localizations: Option<Localisations>,
    description: String,
    description_localizations: Option<Localisations>,
    options: Option<Vec<ApplicationCommandOption>>,
    default_member_permissions: Option<String>,
    dm_permission: Option<bool>,
    default_permission: Option<bool>,
    nsfw: Option<bool>,
    version: Snowflake,
}

#[derive(Debug, Deserialize)]
struct ApplicationCommandInteractionDataOption {}

#[derive(Debug, Deserialize)]
struct InteractionData {
    #[serde(rename = "type")]
    _type: i32,

    id: Snowflake,
    name: String,
    resolved: Option<ResolvedData>,
    options: Option<Vec<ApplicationCommandInteractionDataOption>>,
    focused: Option<bool>,
}

#[derive(Debug)]
pub enum InteractionType {
    Ping,
    ApplicationCommand,
    MessageComponent,
    ApplicationCommandAutocomplete,
    ModalSubmit,
    Unknown,
}

#[derive(Debug, Deserialize)]
pub struct Interaction {
    #[serde(rename = "type")]
    _type: i32,

    id: Snowflake,
    application_id: Snowflake,
    data: Option<InteractionData>,
    guild_id: Option<Snowflake>,
    channel: Option<Channel>,
    channel_id: Option<Snowflake>,
    member: Option<Member>,
    user: Option<User>,
    token: String,
    version: i32,
    message: Option<Message>,
    app_permissions: Option<String>,
    locale: Option<String>,
    guild_locale: Option<String>,
    entitlements: Vec<Entitlement>,
}

impl Interaction {
    pub fn inttype(&self) -> InteractionType {
        match self._type {
            1 => InteractionType::Ping,
            2 => InteractionType::ApplicationCommand,
            3 => InteractionType::MessageComponent,
            4 => InteractionType::ApplicationCommandAutocomplete,
            5 => InteractionType::ModalSubmit,
            _ => InteractionType::Unknown,
        }
    }
}

#[derive(Debug)]
enum InteractionCallbackType {
    Pong,
    ChannelMessageWithSource,
    DeferredChannelMessageWithSource,
    DeferredUpdateMessage,
    UpdateMessage,
    ApplicationCommandAutocompleteResult,
    Modal,
    PremiumRequired,
}

impl InteractionCallbackType {
    fn ordinal(&self) -> i32 {
        match self {
            InteractionCallbackType::Pong => 1,
            InteractionCallbackType::ChannelMessageWithSource => 4,
            InteractionCallbackType::DeferredChannelMessageWithSource => 5,
            InteractionCallbackType::DeferredUpdateMessage => 6,
            InteractionCallbackType::UpdateMessage => 7,
            InteractionCallbackType::ApplicationCommandAutocompleteResult => 8,
            InteractionCallbackType::Modal => 9,
            InteractionCallbackType::PremiumRequired => 10,
        }
    }
}

impl Serialize for InteractionCallbackType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_i32(self.ordinal())
    }
}

#[derive(Debug, Serialize)]
struct InteractionCallbackData {}

#[derive(Debug, Serialize)]
pub struct InteractionResponse {
    #[serde(rename = "type")]
    _type: InteractionCallbackType,
    data: Option<InteractionCallbackData>,
}

impl InteractionResponse {
    pub fn pong() -> Self {
        InteractionResponse {
            _type: InteractionCallbackType::Pong,
            data: None,
        }
    }
}
