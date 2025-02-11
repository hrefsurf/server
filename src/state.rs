#[derive(Clone)]
pub struct AppState {
    pub db_pool: sqlx::MySqlPool,
    pub tera: tera::Tera,
}
