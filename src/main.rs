use crate::{extrato::get_extrato, structs::NovaTransacao, transacao::criar_transacao};
use actix_web::{
    get, post,
    web::{self, Data},
    App, HttpRequest, HttpResponse, HttpServer, Responder,
};
use dotenv::dotenv;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::net::TcpListener;
mod extrato;
mod structs;
pub mod transacao;
use actix_web::error;
use actix_web::web::JsonConfig;

pub struct AppState {
    pub db: PgPool,
}

fn handle_error(error: structs::CustomErrors) -> HttpResponse {
    match error {
        structs::CustomErrors::NotFound => HttpResponse::NotFound().finish(),
        structs::CustomErrors::NoBalance => HttpResponse::UnprocessableEntity().finish(),
        structs::CustomErrors::InvalidInput => HttpResponse::UnprocessableEntity().finish(),
        structs::CustomErrors::Internal => HttpResponse::InternalServerError().finish(),
    }
}

fn json_error_handler(err: error::JsonPayloadError, _req: &HttpRequest) -> error::Error {
    match err {
        error::JsonPayloadError::Deserialize(_) => {
            error::InternalError::from_response(err, HttpResponse::UnprocessableEntity().finish())
                .into()
        }
        _ => err.into(),
    }
}

#[get("/clientes/{id_cliente}/extrato")]
async fn extrato_route(path: web::Path<i32>, state: web::Data<AppState>) -> impl Responder {
    let id_cliente = path.into_inner();

    match get_extrato(id_cliente, state).await {
        Ok(extrato) => HttpResponse::Ok().json(extrato),
        Err(error) => handle_error(error),
    }
}

#[post("/clientes/{id_cliente}/transacoes")]
async fn transacao_route(
    info: web::Json<NovaTransacao>,
    path: web::Path<i32>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id_cliente = path.into_inner();

    match criar_transacao(id_cliente, info.into_inner(), state).await {
        Ok(transacao) => HttpResponse::Ok().json(transacao),
        Err(error) => handle_error(error),
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set.");

    let pool = PgPoolOptions::new()
        .max_connections(25)
        .connect(&database_url)
        .await
        .unwrap();

    let listener = TcpListener::bind("0.0.0.0:3000").expect("Failed to create listener");

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(AppState { db: pool.clone() }))
            .app_data(JsonConfig::default().error_handler(json_error_handler))
            .service(extrato_route)
            .service(transacao_route)
    })
    .listen(listener)
    .expect("fail to bind")
    .run()
    .await?;

    Ok(())
}
