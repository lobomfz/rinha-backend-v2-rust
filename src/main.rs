use actix_web::{
    get,
    web::{self, Data},
    App, HttpServer, Responder,
};
use chrono::Utc;
use dotenv::dotenv;
use serde::Serialize;
use sqlx::{postgres::PgPoolOptions, types::chrono, PgPool};
use std::net::TcpListener;

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

async fn get_extrato(id_cliente: i32, pool: sqlx::PgPool) -> Extrato {
    let saldo = sqlx::query!(
        "SELECT c.saldo as total, c.limite FROM clientes AS c WHERE c.id = $1",
        id_cliente
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    let ultimas_transacoes = sqlx::query_as!(
        Transacao,
        r#"SELECT t.valor, t.tipo as "tipo: TipoTransacao", t.descricao, t.realizada_em
        FROM transacoes AS t
        WHERE t.id_cliente = $1
        ORDER BY t.realizada_em DESC
        LIMIT 10"#,
        id_cliente,
    )
    .fetch_all(&pool)
    .await
    .unwrap();

    return Extrato {
        saldo: ExtratoSaldo {
            total: saldo.total,
            data_extrato: Utc::now(),
            limite: saldo.limite,
        },
        ultimas_transacoes,
    };
}

#[derive(sqlx::FromRow, Debug, Serialize)]
struct ExtratoSaldo {
    total: i32,
    data_extrato: chrono::DateTime<chrono::Utc>,
    limite: i32,
}

#[derive(Debug, sqlx::Type, Serialize)]
#[sqlx(rename_all = "lowercase")]
enum TipoTransacao {
    D,
    C,
}

#[derive(sqlx::FromRow, Debug, Serialize)]
struct Transacao {
    valor: i32,
    tipo: TipoTransacao,
    descricao: String,
    realizada_em: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize)]
struct Extrato {
    saldo: ExtratoSaldo,
    ultimas_transacoes: Vec<Transacao>,
}
