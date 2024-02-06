use actix_web::web;
use chrono::Utc;
use tokio::try_join;

use crate::{
    structs::{CustomErrors, Extrato, ExtratoSaldo, TipoTransacao, Transacao},
    AppState,
};

pub async fn get_extrato(
    id_cliente: i32,
    state: web::Data<AppState>,
) -> Result<Extrato, CustomErrors> {
    let mut transaction = match state.db.begin().await {
        Ok(transaction) => transaction,
        Err(_) => return Err(CustomErrors::Internal),
    };

    let saldo_result = sqlx::query!(
        "SELECT c.saldo as total, c.limite FROM clientes AS c WHERE c.id = $1",
        id_cliente
    )
    .fetch_one(&mut *transaction)
    .await;

    let ultimas_transacoes_result = sqlx::query_as!(
        Transacao,
        r#"SELECT t.valor, t.tipo as "tipo: TipoTransacao", t.descricao, t.realizada_em
        FROM transacoes AS t
        WHERE t.id_cliente = $1
        ORDER BY t.realizada_em DESC
        LIMIT 10"#,
        id_cliente,
    )
    .fetch_all(&mut *transaction)
    .await;

    if saldo_result.is_err() {
        transaction.rollback().await.unwrap();
        return Err(CustomErrors::NotFound);
    }

    match saldo_result {
        Ok(saldo) => {
            let ultimas_transacoes = match ultimas_transacoes_result {
                Ok(ultimas_transacoes) => ultimas_transacoes,
                Err(_) => {
                    transaction.rollback().await.unwrap();
                    return Err(CustomErrors::Internal);
                }
            };

            return Ok(Extrato {
                saldo: ExtratoSaldo {
                    total: saldo.total,
                    data_extrato: Utc::now(),
                    limite: saldo.limite,
                },
                ultimas_transacoes,
            });
        }
        Err(_) => return Err(CustomErrors::NotFound),
    }
}
