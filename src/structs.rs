use serde::{Deserialize, Serialize};

#[derive(sqlx::FromRow, Serialize)]
pub struct ExtratoSaldo {
    pub total: i32,
    pub data_extrato: chrono::DateTime<chrono::Utc>,
    pub limite: i32,
}

#[derive(sqlx::Type, Serialize, Deserialize, PartialEq)]
#[sqlx(type_name = "tipo_transacao", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum TipoTransacao {
    D,
    C,
}

#[derive(sqlx::FromRow, Serialize)]
pub struct Transacao {
    pub valor: i32,
    pub tipo: TipoTransacao,
    pub descricao: String,
    pub realizada_em: chrono::NaiveDateTime,
}

#[derive(Serialize)]
pub struct Extrato {
    pub saldo: ExtratoSaldo,
    pub ultimas_transacoes: Vec<Transacao>,
}

pub enum CustomErrors {
    NotFound,
    NoBalance,
    Internal,
    InvalidInput,
}

// {
//     "valor": 1000,
//     "tipo": "d",
//     "descricao": "teste"
// }
#[derive(sqlx::FromRow, Deserialize, Serialize)]
pub struct NovaTransacao {
    pub valor: i32,
    pub tipo: TipoTransacao,
    pub descricao: String,
}

#[derive(Serialize)]
pub struct NovaTransacaoResponse {
    pub limite: i32,
    pub saldo: i32,
}
