use std::time::Duration;

#[derive(Default)]
pub struct InMemoryResponseCacherOptions {
    ttl: Option<Duration>,
}

impl InMemoryResponseCacherOptions {
    pub fn builder() -> InMemoryResponseCacherOptionsBuilder {
        InMemoryResponseCacherOptionsBuilder::default()
    }

    pub fn ttl(&self) -> Option<Duration> {
        self.ttl
    }
}

#[derive(Default)]
pub struct InMemoryResponseCacherOptionsBuilder {
    ttl: Option<Duration>,
}

impl InMemoryResponseCacherOptionsBuilder {
    pub fn ttl(mut self, ttl: Duration) -> Self {
        self.ttl = Some(ttl);
        self
    }

    pub fn build(self) -> InMemoryResponseCacherOptions {
        InMemoryResponseCacherOptions { ttl: self.ttl }
    }
}
