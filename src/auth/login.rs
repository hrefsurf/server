use axum::response::Html;

#[axum_macros::debug_handler]
pub async fn get(
) -> Html<&'static str> {
    Html("")
}
