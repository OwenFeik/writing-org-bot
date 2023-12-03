use actix_web::{
    error::{ErrorBadRequest, ErrorUnauthorized, ErrorUnprocessableEntity},
    post, web, HttpRequest,
};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    consts::APPLICATION_ID,
    discord::{ApplicationCommand, ApplicationCommandType, InteractionType},
    req::api_uri,
};

mod announcer;
mod auth;
mod consts;
mod csv;
mod discord;
mod req;

#[cfg(test)]
mod test;

type Error = String;
type Result<T> = std::result::Result<T, Error>;

fn msg<S: ToString>(msg: S) -> Error {
    msg.to_string()
}

fn err<T, S: ToString>(message: S) -> Result<T> {
    Err(msg(message))
}

fn extract_header<'a>(req: &'a HttpRequest, header: &str) -> Result<&'a str> {
    match req.headers().get(header) {
        Some(value) => value.to_str().map_err(msg),
        None => err(format!("missing {} header", header)),
    }
}

fn e400<S: ToString>(message: S) -> actix_web::Error {
    ErrorBadRequest(msg(message))
}

fn e422<S: ToString>(message: S) -> actix_web::Error {
    ErrorUnprocessableEntity(msg(message))
}

fn e401<S: ToString>(message: S) -> actix_web::Error {
    ErrorUnauthorized(msg(message))
}

async fn register_commands() {
    #[derive(serde::Serialize)]
    struct ApplicationCommandRequest {
        name: String,
        description: String,

        #[serde(rename = "type")]
        _type: i32,
    }

    let reqs = vec![
        ApplicationCommandRequest {
            name: "announce".to_string(),
            description: "Enable announcing in this channel.".to_string(),
            _type: ApplicationCommandType::ChatInput.ordinal(),
        },
        ApplicationCommandRequest {
            name: "cancel".to_string(),
            description: "Disable announcing in this channel.".to_string(),
            _type: ApplicationCommandType::ChatInput.ordinal(),
        },
    ];

    for req in reqs {
        match req::post::<&str, ApplicationCommandRequest, ApplicationCommand>(
            &api_uri(&format!("/applications/{APPLICATION_ID}/commands")),
            req,
        )
        .await
        {
            Ok(command) => println!("{command:?}"),
            Err(e) => eprintln!("Failed to register command: {e}"),
        }
    }
}

#[post("/api/interactions")]
async fn interactions(
    req: HttpRequest,
    body: web::Bytes,
    commands: web::Data<UnboundedSender<announcer::AnnouncerCommand>>,
) -> std::result::Result<web::Json<discord::InteractionResponse>, actix_web::Error> {
    let body = String::from_utf8(body.to_vec()).map_err(e422)?;
    let sighex = extract_header(&req, "X-Signature-Ed25519").map_err(e400)?;
    let timestamp = extract_header(&req, "X-Signature-Timestamp").map_err(e400)?;
    if !auth::validate(sighex, timestamp, &body).map_err(e400)? {
        return Err(e401("invalid request signature"));
    }

    let interaction = serde_json::de::from_str::<discord::Interaction>(&body).map_err(e422)?;
    dbg!(&interaction);

    let resp = match interaction.inttype() {
        InteractionType::ApplicationCommand => match (interaction.channel(), interaction.command())
        {
            (Some(channel), Some("announce")) => {
                commands
                    .send(announcer::AnnouncerCommand::RegisterChannel(
                        channel.clone(),
                    ))
                    .ok();
                discord::InteractionResponse::message(
                    "Announcements will be sent in this channel every Sunday morning.",
                )
            }
            (Some(channel), Some("cancel")) => {
                commands
                    .send(announcer::AnnouncerCommand::UnregisterChannel(
                        channel.clone(),
                    ))
                    .ok();
                discord::InteractionResponse::message(
                    "Announcements will not longer be sent in this channel.",
                )
            }
            (None, _) => {
                discord::InteractionResponse::message("Use this command in a server channel.")
            }
            _ => discord::InteractionResponse::message("Unrecognised command."),
        },
        InteractionType::Ping => discord::InteractionResponse::pong(),
        _ => return Err(e422("unhandled interaction type")),
    };

    Ok(web::Json(resp))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    register_commands().await;

    let (send, recv) = tokio::sync::mpsc::unbounded_channel();
    announcer::run_announcer(recv).await;

    actix_web::HttpServer::new(move || {
        let commands = web::Data::new(send.clone());

        actix_web::App::new()
            .wrap(actix_web::middleware::Logger::default())
            .app_data(commands)
            .service(interactions)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
