use actix_web::{dev::Service, post, web, Responder};

mod auth;
mod discord;

#[post("/api/interactions")]
async fn interactions(interaction: web::Json<discord::Interaction>) -> impl Responder {
    dbg!(interaction);
    web::Json(discord::InteractionResponse::pong())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    actix_web::HttpServer::new(|| {
        actix_web::App::new()
            .wrap_fn(|req, srv| {
                req.headers().get("X-Signature-Ed25519");

                let fut = srv.call(req);
                async {
                    let res = fut.await?;
                    Ok(res)
                }
            })
            .service(interactions)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
