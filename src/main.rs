mod api;
mod core;
mod traits;

use actix_cors::Cors;
use actix_web::{
    http,
    web::{self, Data},
    App, HttpServer,
};
use api::routes::notebook_routes;
use api::state::State;
use dotenv::dotenv;
use tracing::info;
use tracing_subscriber;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();
    dotenv().ok();

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .expect("PORT must be a number");

    let data = Data::new(State::new());

    info!("Starting server on port {}", port);
    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:5173")
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
    })
    .bind(("127.0.0.1", port))?
    .run()
    .await
}
