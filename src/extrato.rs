use chrono::Utc;
use tokio::try_join;

use crate::structs::{CustomErrors, Extrato, ExtratoSaldo, TipoTransacao, Transacao};

pub async fn get_extrato(id_cliente: i32, pool: sqlx::PgPool) -> Result<Extrato, CustomErrors> {
    let saldo_future = sqlx::query!(
        "SELECT c.saldo as total, c.limite FROM clientes AS c WHERE c.id = $1",
        id_cliente
    )
    .fetch_one(&pool);

    let ultimas_transacoes_future = sqlx::query_as!(
        Transacao,
        r#"SELECT t.valor, t.tipo as "tipo: TipoTransacao", t.descricao, t.realizada_em
        FROM transacoes AS t
        WHERE t.id_cliente = $1
        ORDER BY t.realizada_em DESC
        LIMIT 10"#,
        id_cliente,
    )
    .fetch_all(&pool);

    let query_result = try_join!(saldo_future, ultimas_transacoes_future);

    match query_result {
        Ok((saldo, ultimas_transacoes)) => {
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
