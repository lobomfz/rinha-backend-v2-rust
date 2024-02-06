use crate::extrato::get_extrato;
use actix_web::{
    get,
    web::{self, Data},
    App, HttpResponse, HttpServer, Responder,
};
use dotenv::dotenv;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::net::TcpListener;
mod extrato;
mod structs;

#[get("/clientes/{id_cliente}/extrato")]
async fn one(path: web::Path<i32>, pool: web::Data<PgPool>) -> impl Responder {
    let id_cliente = path.into_inner();

    match get_extrato(id_cliente, pool.get_ref().clone()).await {
        Ok(extrato) => HttpResponse::Ok().json(extrato),
        Err(error) => match error {
            structs::CustomErrors::NotFound => HttpResponse::NotFound().finish(),
        },
    }
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
