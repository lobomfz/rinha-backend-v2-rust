use actix_web::{
    get,
    web::{self, Data},
    App, HttpServer, Responder,
};
use dotenv::dotenv;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::net::TcpListener;

use crate::extrato::get_extrato;

mod extrato;
mod structs;

#[get("/extrato")]
async fn one(pool: web::Data<PgPool>) -> impl Responder {
    let extrato = get_extrato(1, pool.get_ref().clone()).await;

    return web::Json(extrato);
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set.");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .unwrap();

    let listener = TcpListener::bind("0.0.0.0:4444").expect("Failed to create listener");

    HttpServer::new(move || App::new().app_data(Data::new(pool.clone())).service(one))
        .listen(listener)
        .expect("fail to bind")
        .run()
        .await?;

    Ok(())
}
