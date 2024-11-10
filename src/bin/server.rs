use actix_web::{get, middleware, App, HttpServer, Responder, HttpResponse};
use actix_files as fs;
use actix_cors::Cors;
use log::info;
use serde::Serialize;
use std::env;

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
    let frontend_origin = env::var("FRONTEND_ORIGIN")
        .unwrap_or_else(|_| "http://localhost:5173".to_string());
    
    info!("starting at http://localhost:{}", port);
    info!("frontend origin: {}", frontend_origin);

    HttpServer::new(move || {
        // CORS configuration for development
        let cors = Cors::default()
            .allowed_origin(&frontend_origin)
            .allow_any_method()
            .allow_any_header()
            .supports_credentials();

        App::new()
            .wrap(middleware::Logger::default())
            .wrap(cors)
            // API routes
            .service(hello)
            // WebSocket route
            .service(lib::server::listen)
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
    .bind(format!("127.0.0.1:{}", port))?
    .run()
    .await
}