use actix_web::{App, HttpResponse, HttpServer, web};
use actix_web::dev::Server;
use anyhow::Result;

async fn health_check() -> HttpResponse {
    HttpResponse::Ok().finish()
}

pub fn run() -> Result<Server> {
    let server = HttpServer::new(|| {
        App::new()
            .route("/health_check", web::get().to(health_check))
    })
        .bind("127.0.0.1:8000")?
        .run();
    Ok(server)
}
