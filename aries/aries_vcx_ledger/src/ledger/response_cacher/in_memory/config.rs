use std::{num::NonZeroUsize, time::Duration};

use serde::Deserialize;

use crate::errors::error::VcxLedgerError;

#[derive(Clone, Debug, Deserialize)]
pub struct InMemoryResponseCacherConfig {
    ttl: Duration,
    capacity: NonZeroUsize,
}

impl InMemoryResponseCacherConfig {
    pub fn builder() -> InMemoryResponseCacherConfigBuilder {
        InMemoryResponseCacherConfigBuilder::default()
    }

    pub fn ttl(&self) -> Duration {
        self.ttl
    }

    pub fn capacity(&self) -> NonZeroUsize {
        self.capacity
    }
}

#[derive(Default)]
pub struct InMemoryResponseCacherConfigBuilder {}

pub struct InMemoryResponseCacherConfigBuilderTtlSet {
    ttl: Duration,
}

pub struct InMemoryResponseCacherConfigBuilderReady {
    ttl: Duration,
    capacity: NonZeroUsize,
}

impl InMemoryResponseCacherConfigBuilder {
    pub fn ttl(self, ttl: Duration) -> InMemoryResponseCacherConfigBuilderTtlSet {
        InMemoryResponseCacherConfigBuilderTtlSet { ttl }
    }
}

impl InMemoryResponseCacherConfigBuilderTtlSet {
    pub fn capacity(
        self,
        capacity: usize,
    ) -> Result<InMemoryResponseCacherConfigBuilderReady, VcxLedgerError> {
        let capacity = NonZeroUsize::new(capacity).ok_or(VcxLedgerError::InvalidOption(
            "Failed to parse cache capacity into NonZeroUsize".into(),
        ))?;
        Ok(InMemoryResponseCacherConfigBuilderReady {
            ttl: self.ttl,
            capacity,
        })
    }
}

impl InMemoryResponseCacherConfigBuilderReady {
    pub fn build(self) -> InMemoryResponseCacherConfig {
        InMemoryResponseCacherConfig {
            ttl: self.ttl,
            capacity: self.capacity,
        }
    }
}
