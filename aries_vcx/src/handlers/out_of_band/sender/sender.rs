use crate::handlers::out_of_band::{OutOfBand, GoalCode};
use crate::messages::attachment::{AttachmentId, AttachmentEncoding};
use crate::messages::connection::service::ServiceResolvable;
use crate::error::prelude::*;

impl OutOfBand {
    pub fn create() -> Self {
        Self::default()
    }

    pub fn set_label(mut self, label: &str) -> Self {
        self.label = Some(label.to_string());
        self
    }

    pub fn set_goal_code(mut self, goal_code: GoalCode) -> Self {
        self.goal_code = Some(goal_code);
        self
    }

    pub fn set_goal(mut self, goal: &str) -> Self {
        self.goal = Some(goal.to_string());
        self
    }

    pub fn append_request(mut self, attach_id: AttachmentId, attach: &str) -> VcxResult<Self> {
        self.requests_attach.add_json_attachment(attach_id, ::serde_json::Value::String(attach.to_string()), AttachmentEncoding::Json)?;
        Ok(self)
    }

    pub fn append_service(mut self, service: ServiceResolvable) -> Self {
        let services = self.services.push(service);
        self
    }
}
