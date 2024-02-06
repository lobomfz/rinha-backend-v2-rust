use crate::structs::{CustomErrors, NovaTransacao, NovaTransacaoResponse, TipoTransacao};

pub async fn criar_transacao(
    id_cliente: i32,
    transacao: NovaTransacao,
    pool: sqlx::PgPool,
) -> Result<NovaTransacaoResponse, CustomErrors> {
    // o quão esperto é esse compiler?
    if transacao.descricao.len() > 10 || transacao.descricao.len() < 1 {
        return Err(CustomErrors::InvalidInput);
    }

    let transaction = pool.begin().await.unwrap();

    let is_debito = transacao.tipo == TipoTransacao::D;

    if is_debito {
        let result = sqlx::query!(
            "SELECT c.saldo, c.limite FROM clientes as c WHERE id = $1",
            id_cliente
        )
        .fetch_optional(&pool)
        .await
        .unwrap();

        match result {
            Some(cliente) => {
                if cliente.saldo - transacao.valor < -cliente.limite {
                    transaction.rollback().await.unwrap();
                    return Err(CustomErrors::NoBalance);
                }
            }
            None => {
                println!("Cliente não encontrado");
                transaction.rollback().await.unwrap();
                return Err(CustomErrors::NotFound);
            }
        }
    }

    let valor = if is_debito {
        -transacao.valor
    } else {
        transacao.valor
    };

    let cliente_future = sqlx::query!(
        "UPDATE clientes SET saldo = saldo + $1 WHERE id = $2 RETURNING limite, saldo",
        valor,
        id_cliente,
    )
    .fetch_one(&pool);

    let transacao_future = sqlx::query!(
        "INSERT INTO transacoes (id_cliente, valor, tipo, descricao) VALUES ($1, $2, $3, $4)",
        id_cliente,
        transacao.valor,
        transacao.tipo as _,
        transacao.descricao
    )
    .execute(&pool);

    let result = futures::try_join!(cliente_future, transacao_future);

    match result {
        Ok((cliente, _)) => {
            if is_debito && cliente.saldo < -cliente.limite {
                transaction.rollback().await.unwrap();
                return Err(CustomErrors::NoBalance);
            }

            transaction.commit().await.unwrap();

            return Ok(NovaTransacaoResponse {
                limite: cliente.limite,
                saldo: cliente.saldo,
            });
        }
        Err(err) => {
            transaction.rollback().await.unwrap();
            println!("Erro ao criar transação: {:?}", err);
            return Err(CustomErrors::Internal);
        }
    }
}
