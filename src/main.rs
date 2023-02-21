mod api;
mod core;

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
use tracing::info;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();
    dotenv().ok();

    let api_port = std::env::var("API_PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .expect("PORT must be a number");
    let zmq_port = std::env::var("ZMQ_PORT")
        .unwrap_or_else(|_| "80801".to_string())
        .parse::<u16>()
        .expect("ZMQ_PORT must be a number");
    let client_url = std::env::var("CLIENT_URL").expect("CLIENT_URL must be set");

    info!("ZMQ port {}", zmq_port);
    let ctx = zmq::Context::new();
    let subscriber = ctx.socket(zmq::SUB).expect("Could not create socket");

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
