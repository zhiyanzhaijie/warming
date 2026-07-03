#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApiError {
    pub message: String,
}

pub type ApiResult<T> = Result<T, ApiError>;

impl ApiError {
    pub fn new(message: impl ToString) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for ApiError {}

impl From<app::app_error::AppError> for ApiError {
    fn from(err: app::app_error::AppError) -> Self {
        Self::new(err)
    }
}
