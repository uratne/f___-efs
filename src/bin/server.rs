use actix_web::{get, middleware, web, App, HttpResponse, HttpServer, Responder};
use actix_files as fs;
use actix_cors::Cors;
use lib::server::broadcaster;
use log::info;
use serde::Serialize;
use std::{env, sync::Arc};

#[derive(Serialize)]
struct ApiResponse {
    message: String,
}

#[get("/api/hello")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().json(ApiResponse {
        message: "Hello from Rust!".to_string(),
    })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let host = env::var("HOST").unwrap_or_else(|_| "localhost".to_string());
    let frontend_origin = env::var("FRONTEND_ORIGIN")
        .unwrap_or_else(|_| "http://localhost:5173".to_string());
    
    info!("starting at http://{}:{}", host, port);
    info!("frontend origin: {}", frontend_origin);

    let broadcasters = broadcaster::new_broadcasters();
    let broadcasters = Arc::new(broadcasters);

    HttpServer::new(move || {
        // CORS configuration for development
        let cors = Cors::default()
            .allowed_origin(&frontend_origin)
            .allow_any_method()
            .allow_any_header()
            .supports_credentials();
        let broadcasters = Arc::clone(&broadcasters);
        let broadcasters = web::Data::new(broadcasters);

        App::new()
            .wrap(middleware::Logger::default())
            .wrap(cors)
            .app_data(broadcasters)
            // API routes
            .service(hello)
            // WebSocket route
            .service(lib::server::data_inbound)
            // SSE route
            .service(lib::server::data_outbound)
            // In production, serve the built frontend
            .service(
                fs::Files::new("/", "./frontend/build")
                    .index_file("index.html")
                    .default_handler(|req: actix_web::dev::ServiceRequest| {
                        let (http_req, _payload) = req.into_parts();
                        async {
                            let response = fs::NamedFile::open("./frontend/build/index.html")?
                                .into_response(&http_req);
                            Ok(actix_web::dev::ServiceResponse::new(http_req, response))
                        }
                    })
            )
    })
    .bind(format!("{}:{}", host, port))?
    .run()
    .await
}