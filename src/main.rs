mod api;
mod core;
mod kernel;

use actix_cors::Cors;
use actix_web::{
    http,
    web::{self, Data},
    App, HttpServer,
};
use api::{
    routes::{notebook_routes, ws_routes},
    state::State,
};
use dotenv::dotenv;
use kernel::kernel_client::{KernelClient, KernelMessage};
use std::collections::HashMap;
use tracing::info;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();
    dotenv().ok();

    let api_port = std::env::var("API_PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .expect("PORT must be a number");
    let client_url = std::env::var("CLIENT_URL").expect("CLIENT_URL must be set");

    let kernel_client = KernelClient::new().expect("Could not create kernel client");
    let msg = KernelMessage {
        content: "print(123)\na = 1 + 2".to_string(),
        locals: HashMap::new(),
    };
    kernel_client
        .send_to_kernel(&msg)
        .expect("Could not send message");

    info!("Starting server on port {}", api_port);
    let data = Data::new(State::new());
    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin(&client_url)
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
            .allowed_header(http::header::CONTENT_TYPE)
            .max_age(3600);

        App::new()
            .app_data(Data::clone(&data))
            .wrap(cors)
            .wrap(actix_web::middleware::Logger::new(
                "Router: %r, Status: %s, Time: %Dms",
            ))
            .service(web::scope("/api").configure(notebook_routes))
            .service(web::scope("/ws").configure(ws_routes))
    })
    .bind(("127.0.0.1", api_port))?
    .run()
    .await
}
