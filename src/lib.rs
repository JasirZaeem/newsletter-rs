use actix_web::{App, HttpResponse, HttpServer, web};
use actix_web::dev::Server;
use anyhow::Result;
use std::net::TcpListener;

async fn health_check() -> HttpResponse {
    HttpResponse::Ok().finish()
}

pub fn run(listener: TcpListener) -> Result<Server> {
    let server = HttpServer::new(|| {
        App::new()
            .route("/health_check", web::get().to(health_check))
    })
        .listen(listener)?
        .run();
    Ok(server)
}
