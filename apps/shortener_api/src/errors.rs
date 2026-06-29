use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use tracing::error;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to connect to Postgres")]
    PostgresConnect(#[source] sqlx::Error),

    #[error("database query failed")]
    Database(#[source] sqlx::Error),

    #[error("failed to generate a unique short code after {attempts} attempts")]
    CodeGenerationExhausted { attempts: usize },

    #[error("url does not exist")]
    RedirectNotFound,

    #[error("self-referential URL")]
    SelfReferentialUrl,

    #[error("unsupported URL scheme")]
    UnsupportedUrlScheme,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: ErrorBody,
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    message: &'static str,
}

impl Error {
    fn status(&self) -> StatusCode {
        match self {
            Self::PostgresConnect(_) | Self::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::CodeGenerationExhausted { .. } => StatusCode::SERVICE_UNAVAILABLE,
            Self::RedirectNotFound => StatusCode::NOT_FOUND,
            Self::SelfReferentialUrl | Self::UnsupportedUrlScheme => StatusCode::BAD_REQUEST,
        }
    }

    fn message(&self) -> &'static str {
        match self {
            Self::PostgresConnect(_) | Self::Database(_) => "Unable to shorten this URL.",
            Self::CodeGenerationExhausted { .. } => "Unable to shorten this URL right now.",
            Self::RedirectNotFound => "This short link does not exist.",
            Self::SelfReferentialUrl => "Short links cannot point back to this service.",
            Self::UnsupportedUrlScheme => "Only http:// and https:// URLs can be shortened.",
        }
    }

    fn log(&self) {
        match self {
            Self::PostgresConnect(error) => error!(?error, "Postgres connection failed"),
            Self::Database(error) => error!(?error, "database query failed"),
            Self::CodeGenerationExhausted { attempts } => {
                error!(attempts, "failed to generate unique short code");
            }
            Self::RedirectNotFound | Self::SelfReferentialUrl | Self::UnsupportedUrlScheme => {}
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        self.log();

        let body = ErrorResponse {
            error: ErrorBody {
                message: self.message(),
            },
        };

        (self.status(), Json(body)).into_response()
    }
}
