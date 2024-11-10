use actix_web::{middleware, App, HttpServer};
use actix_files as fs;
use log::info;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    info!("Starting server at http://127.0.0.1:8080");
    
    HttpServer::new(|| {
        App::new()
            // Enable logger middleware
            .wrap(middleware::Logger::default())
            // Serve static files from the "static" directory
            .service(fs::Files::new("/", "./static").index_file("index.html"))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}