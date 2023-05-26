use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct ExtraFieldsSov {
    #[serde(default)]
    priority: u32,
    #[serde(default)]
    recipient_keys: Vec<String>,
    #[serde(default)]
    routing_keys: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    accept: Vec<String>,
}

impl ExtraFieldsSov {
    pub fn builder() -> ExtraFieldsSovBuilder {
        ExtraFieldsSovBuilder::default()
    }

    pub fn priority(&self) -> u32 {
        self.priority
    }

    pub fn recipient_keys(&self) -> &[String] {
        self.recipient_keys.as_ref()
    }

    pub fn routing_keys(&self) -> &[String] {
        self.routing_keys.as_ref()
    }

    pub fn accept(&self) -> &[String] {
        self.accept.as_ref()
    }
}

#[derive(Default)]
pub struct ExtraFieldsSovBuilder {
    priority: u32,
    recipient_keys: Vec<String>,
    routing_keys: Vec<String>,
    accept: Vec<String>,
}

impl ExtraFieldsSovBuilder {
    pub fn set_priority(&mut self, priority: u32) -> &mut Self {
        self.priority = priority;
        self
    }

    pub fn set_recipient_keys(&mut self, recipient_keys: Vec<String>) -> &mut Self {
        self.recipient_keys = recipient_keys;
        self
    }

    pub fn add_recipient_key(&mut self, recipient_key: String) -> &mut Self {
        self.recipient_keys.push(recipient_key);
        self
    }

    pub fn set_routing_keys(&mut self, routing_keys: Vec<String>) -> &mut Self {
        self.routing_keys = routing_keys;
        self
    }

    pub fn add_routing_key(&mut self, routing_key: String) -> &mut Self {
        self.routing_keys.push(routing_key);
        self
    }

    pub fn set_accept(&mut self, accept: Vec<String>) -> &mut Self {
        self.accept = accept;
        self
    }

    pub fn add_accept(&mut self, accept: String) -> &mut Self {
        self.accept.push(accept);
        self
    }

    pub fn build(&self) -> ExtraFieldsSov {
        ExtraFieldsSov {
            priority: self.priority,
            recipient_keys: self.recipient_keys.clone(),
            routing_keys: self.routing_keys.clone(),
            accept: self.accept.clone(),
        }
    }
}
