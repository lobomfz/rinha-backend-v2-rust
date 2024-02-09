use actix_web::web;

use crate::{
    structs::{CustomErrors, NovaTransacao, NovaTransacaoResponse, TipoTransacao},
    AppState,
};

pub async fn criar_transacao(
    id_cliente: i32,
    transacao: NovaTransacao,
    state: web::Data<AppState>,
) -> Result<NovaTransacaoResponse, CustomErrors> {
    let len = transacao.descricao.len();

    if len > 10 || len < 1 {
        return Err(CustomErrors::InvalidInput);
    }

    let mut transaction = match state.db.begin().await {
        Ok(transaction) => transaction,
        Err(_) => return Err(CustomErrors::Internal),
    };

    let is_debito = transacao.tipo == TipoTransacao::D;

    let valor = if is_debito {
        -transacao.valor
    } else {
        transacao.valor
    };

    let cliente_result = sqlx::query!(
        "UPDATE clientes SET saldo = saldo + $1 WHERE id = $2 RETURNING limite, saldo",
        valor,
        id_cliente,
    )
    .fetch_one(&mut *transaction)
    .await;

    let cliente = match cliente_result {
        Ok(cliente) => {
            if is_debito && cliente.saldo < -cliente.limite {
                transaction.rollback().await.unwrap();
                return Err(CustomErrors::NoBalance);
            }
            cliente
        }
        Err(err) => {
            transaction.rollback().await.unwrap();
            println!("Erro ao criar transação: {:?}", err);
            return Err(CustomErrors::Internal);
        }
    };

    let transacao_result = sqlx::query!(
        "INSERT INTO transacoes (id_cliente, valor, tipo, descricao) VALUES ($1, $2, $3, $4)",
        id_cliente,
        transacao.valor,
        transacao.tipo as _,
        transacao.descricao
    )
    .execute(&mut *transaction)
    .await;

    match transacao_result {
        Ok(_) => {
            transaction.commit().await.unwrap();
            Ok(NovaTransacaoResponse {
                limite: cliente.limite,
                saldo: cliente.saldo,
            })
        }
        Err(err) => {
            transaction.rollback().await.unwrap();
            println!("Erro ao criar transação: {:?}", err);
            Err(CustomErrors::Internal)
        }
    }
}
