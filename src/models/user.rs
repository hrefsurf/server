
use chrono::NaiveDateTime;

pub struct User {
    pub id: uuid::Uuid,
    pub username: String,
    pub email: String,
    pub description: String,
    pub created: NaiveDateTime
}
