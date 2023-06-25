use axum::body::BoxBody;
use axum::http::{Response, StatusCode};
use axum::response::{IntoResponse, Redirect};
use axum::Json;
use std::borrow::Cow;
use std::collections::HashMap;

pub type Result<T, E = Error> = std::result::Result<T, E>;

/// A common error type that can be used throughout the API.
///
/// Can be returned in a `Result` from an API handler function.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Return `403 Forbidden`
    #[error("user may not perform that action")]
    #[allow(dead_code)]
    Forbidden,

    /// Return `404 Not Found`
    #[error("`{0}` not found")]
    #[allow(dead_code)]
    NotFound(&'static str),

    /// Return `500 Internal Server Error`
    #[error("An error occured: `{0}`")]
    #[allow(dead_code)]
    ServerError(&'static str),

    /// Return `422 Unprocessable Entity`
    ///
    /// This also serializes the `errors` map to JSON
    #[error("error in the request body")]
    UnprocessableEntity {
        errors: HashMap<Cow<'static, str>, Vec<Cow<'static, str>>>,
    },

    /// Return `500 Internal Server Error` on a `anyhow::Error`.
    ///
    /// Via the generated `From<anyhow::Error> for Error` impl, this allows the
    /// use of `?` in handler functions to automatically convert `anyhow::Error` into a response.
    #[error("an internal server error occurred")]
    Anyhow(#[from] anyhow::Error),

    #[error("an internal server error occurred")]
    Lock(#[from] tokio::sync::TryLockError),
}

impl Error {
    /// Convenient constructor for `Error::UnprocessableEntity`.
    ///
    /// Multiple for the same key are collected into a list for that key.
    pub fn unprocessable_entity<K, V>(errors: impl IntoIterator<Item = (K, V)>) -> Self
    where
        K: Into<Cow<'static, str>>,
        V: Into<Cow<'static, str>>,
    {
        let mut error_map = HashMap::new();

        for (key, val) in errors {
            error_map
                .entry(key.into())
                .or_insert_with(Vec::new)
                .push(val.into());
        }

        Self::UnprocessableEntity { errors: error_map }
    }

    fn status_code(&self) -> StatusCode {
        match self {
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::NotFound { .. } => StatusCode::NOT_FOUND,
            Self::UnprocessableEntity { .. } => StatusCode::UNPROCESSABLE_ENTITY,
            Self::ServerError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Anyhow(_) | Self::Lock(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response<BoxBody> {
        match self {
            Self::NotFound(_) => return Redirect::to("/404").into_response(),
            Self::ServerError(_) => return Redirect::to("/404").into_response(),
            Self::UnprocessableEntity { errors } => {
                #[derive(serde::Serialize)]
                struct Errors {
                    errors: HashMap<Cow<'static, str>, Vec<Cow<'static, str>>>,
                }

                return (StatusCode::UNPROCESSABLE_ENTITY, Json(Errors { errors })).into_response();
            }

            Self::Anyhow(ref e) => {
                tracing::error!("Generic error: {:?}", e);
            }
            // Other errors get mapped normally.
            _ => (),
        }

        (self.status_code(), self.to_string()).into_response()
    }
}
