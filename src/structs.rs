use serde::Serialize;

#[derive(sqlx::FromRow, Serialize)]
pub struct ExtratoSaldo {
    pub total: i32,
    pub data_extrato: chrono::DateTime<chrono::Utc>,
    pub limite: i32,
}

#[derive(sqlx::Type, Serialize)]
#[sqlx(rename_all = "lowercase")]
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
