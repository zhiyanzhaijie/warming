use domain::learning::event::Practiced;

use crate::app_error::AppResult;

#[derive(Clone, Default)]
pub struct LearningEventHandler;

impl LearningEventHandler {
    pub fn new() -> Self {
        Self
    }

    pub async fn handle_practiced(&self, _event: &Practiced) -> AppResult<()> {
        Ok(())
    }
}
