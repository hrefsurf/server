use chrono::NaiveDateTime;

pub struct AuthenticatedUserRecord {
    pub user_id: uuid::Uuid,
    pub pass_hash: String,
    pub salt: String,
    pub stale: bool,

    pub updated: NaiveDateTime,
}