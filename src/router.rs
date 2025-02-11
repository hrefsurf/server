use axum::{extract::State, response::Html, Router};
use tera::Context;
use tower_http::{services::ServeDir, trace::TraceLayer};

use crate::{auth::auth_router, state::AppState};

async fn fallback(
    State(state): State<AppState>
) -> Html<String> {
    let context = Context::new();

    Html(
        state
            .tera
            .render("layout.html", &context)
            .unwrap()
    )
}

pub fn build_router() -> Router<AppState> {
    let mut router = Router::new()
        .nest("/auth", auth_router());

    if cfg!(feature = "serve_resources") {
        router = router.nest_service("/res", ServeDir::new("res"));
    }

    router = router
        .fallback(fallback)
        // below line is needed in order to enable logging axum request/responses
        .layer(TraceLayer::new_for_http());

    router
}
