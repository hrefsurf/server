use axum::{http::StatusCode, response::IntoResponse};

pub enum HandlerErrors {
    TeraRenderError(tera::Error)
}

impl IntoResponse for HandlerErrors {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::TeraRenderError(err) => {
                let err_str = err.to_string();
                (StatusCode::INTERNAL_SERVER_ERROR, err_str).into_response()
            }
        }
    }
}

impl From<tera::Error> for HandlerErrors {
    fn from(err: tera::Error) -> Self {
        Self::TeraRenderError(err)
    }
}
