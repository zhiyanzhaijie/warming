use dioxus::prelude::{try_consume_context, use_context_provider};
use std::sync::Arc;
use tokio::sync::OnceCell;

use crate::{ApiError, ApiResult};

#[derive(Clone)]
pub struct LocalAppState(pub Arc<infra::setup::AppState>);

static FALLBACK_STATE: OnceCell<LocalAppState> = OnceCell::const_new();

pub fn use_local_app_state_provider(state: LocalAppState) -> LocalAppState {
    use_context_provider(|| state)
}

pub async fn init() -> ApiResult<LocalAppState> {
    let container = infra::setup::init_app_container(default_database_url())
        .await
        .map_err(ApiError::from)?;

    Ok(LocalAppState(Arc::new(container)))
}

pub async fn state() -> ApiResult<LocalAppState> {
    if let Some(state) = try_consume_context::<LocalAppState>() {
        return Ok(state);
    }

    FALLBACK_STATE.get_or_try_init(init).await.cloned()
}

pub fn default_database_url() -> &'static str {
    "sqlite://warming.sqlite3"
}
