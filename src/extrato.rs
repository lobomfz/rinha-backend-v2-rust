use actix_web::web;
use chrono::Utc;

use crate::{
    structs::{CustomErrors, Extrato, ExtratoSaldo, TipoTransacao, Transacao},
    AppState,
};

pub async fn get_extrato(
    id_cliente: i32,
    state: web::Data<AppState>,
) -> Result<Extrato, CustomErrors> {
    let saldo_result = sqlx::query!(
        "SELECT c.saldo as total, c.limite FROM clientes AS c WHERE c.id = $1",
        id_cliente
    )
    .fetch_one(&state.db)
    .await;

    let saldo = match saldo_result {
        Ok(ref saldo) => saldo,
        Err(_) => {
            return Err(CustomErrors::NotFound);
        }
    };

    let ultimas_transacoes_result = sqlx::query_as!(
        Transacao,
        r#"SELECT t.valor, t.tipo as "tipo: TipoTransacao", t.descricao, t.realizada_em
        FROM transacoes AS t
        WHERE t.id_cliente = $1
        ORDER BY t.realizada_em DESC
        LIMIT 10"#,
        id_cliente,
    )
    .fetch_all(&state.db)
    .await;

    match ultimas_transacoes_result {
        Ok(ultimas_transacoes) => Ok(Extrato {
            saldo: ExtratoSaldo {
                total: saldo.total,
                data_extrato: Utc::now(),
                limite: saldo.limite,
            },
            ultimas_transacoes,
        }),
        Err(err) => {
            println!("Erro ao buscar extrato: {:?}", err);
            return Err(CustomErrors::Internal);
        }
    }
}
