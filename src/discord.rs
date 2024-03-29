use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub type Snowflake = String;

type Timestamp = String;

#[derive(Debug, Deserialize)]
struct Channel {}

#[derive(Debug, Deserialize)]
struct User {}

#[derive(Debug, Deserialize)]
struct Member {}

#[derive(Debug, Deserialize)]
struct Role {}

#[derive(Debug, Deserialize)]
struct ChannelMention {}

#[derive(Debug, Deserialize)]
struct Reaction {}

#[derive(Debug, Deserialize)]
struct MessageActivity {}

#[derive(Debug, Deserialize)]
pub struct Message {
    id: Snowflake,
    channel_id: Snowflake,
    author: User,
    content: String,
    timestamp: Timestamp,
    edited_timestamp: Option<Timestamp>,
    tts: bool,
    mention_everyone: bool,
    mentions: Vec<User>,
    mention_roles: Vec<Role>,
    mention_channels: Option<Vec<ChannelMention>>,
    attachments: Vec<Attachment>,
    embeds: Vec<Embed>,
    reactions: Option<Vec<Reaction>>,
    nonce: Option<String>,
    pinned: bool,
    webhook_id: Option<Snowflake>,

    #[serde(rename = "type")]
    _type: i32,

    activity: Option<MessageActivity>,
    application_id: Option<Snowflake>,
    message_reference: Option<MessageReference>,
    flags: Option<i32>,
    referenced_message: Option<Box<Message>>,
    thread: Option<Channel>,
    components: Option<Vec<MessageComponent>>,
    position: Option<i32>,
}

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

    pub fn channel(&self) -> Option<&Snowflake> {
        self.channel_id.as_ref()
    }

    pub fn command(&self) -> Option<&str> {
        self.data.as_ref().map(|data| data.name.as_str())
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

#[derive(Clone, Debug, Deserialize, Serialize)]
struct EmbedFooter {}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct EmbedImage {}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct EmbedThumbnail {}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct EmbedVideo {}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct EmbedProvider {}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct EmbedAuthor {}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct EmbedField {
    name: String,
    value: String,
    inline: Option<bool>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Embed {
    #[serde(rename = "type")]
    _type: Option<String>,

    title: Option<String>,
    description: Option<String>,
    url: Option<String>,
    timestamp: Option<Timestamp>,
    colour: Option<i32>,
    footer: Option<EmbedFooter>,
    image: Option<EmbedImage>,
    thumbnail: Option<EmbedThumbnail>,
    video: Option<EmbedVideo>,
    provider: Option<EmbedProvider>,
    author: Option<EmbedAuthor>,
    fields: Option<Vec<EmbedField>>,
}

impl Embed {
    const COLOUR: i32 = 0xC84200;

    pub fn new<S: ToString>(title: S, description: S) -> Self {
        Self {
            title: Some(title.to_string()),
            description: Some(description.to_string()),
            colour: Some(Self::COLOUR),
            ..Default::default()
        }
    }

    pub fn add_field(&mut self, name: String, value: String) {
        let field = EmbedField {
            name,
            value,
            inline: None,
        };

        if let Some(fields) = &mut self.fields {
            fields.push(field);
        } else {
            self.fields = Some(vec![field]);
        }
    }
}

#[derive(Debug, Serialize)]
pub struct AllowedMentions {}

#[derive(Debug, Deserialize, Serialize)]
pub struct MessageReference {}

#[derive(Debug, Deserialize, Serialize)]
pub struct MessageComponent {}

#[derive(Debug, Deserialize, Serialize)]
pub struct Attachment {}

#[derive(Debug, Serialize)]
#[serde(untagged)]
enum InteractionCallbackData {
    Message {
        tts: Option<bool>,
        content: Option<String>,
        embeds: Option<Vec<Embed>>,
        allowed_mentions: Option<AllowedMentions>,
        flags: Option<i32>,
        components: Option<Vec<MessageComponent>>,
        attachments: Option<Vec<Attachment>>,
    },
}

impl InteractionCallbackData {
    fn message(content: String) -> Self {
        Self::Message {
            tts: Some(false),
            content: Some(content),
            embeds: None,
            allowed_mentions: None,
            flags: None,
            components: None,
            attachments: None,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct InteractionResponse {
    #[serde(rename = "type")]
    _type: i32,
    data: Option<InteractionCallbackData>,
}

impl InteractionResponse {
    fn new(inttype: InteractionCallbackType, data: Option<InteractionCallbackData>) -> Self {
        Self {
            _type: inttype.ordinal(),
            data,
        }
    }

    pub fn pong() -> Self {
        Self::new(InteractionCallbackType::Pong, None)
    }

    pub fn message<S: ToString>(content: S) -> Self {
        Self::new(
            InteractionCallbackType::ChannelMessageWithSource,
            Some(InteractionCallbackData::message(content.to_string())),
        )
    }
}

#[derive(Debug, Deserialize)]
pub struct ErrorResponse {
    pub message: String,
    code: i32,
}
