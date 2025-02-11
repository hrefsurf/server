use axum::{routing::{get, post}, Router};

mod login;
mod signup;

use crate::state::AppState;

pub fn auth_router() -> Router<AppState> {
    Router::new()
        .route("/login", get(login::get))
        .route("/signup", get(signup::get))
        .route("/signup", post(signup::post))
}
