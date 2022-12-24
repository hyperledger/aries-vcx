#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct PleaseAck {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    on: Vec<AckOn>,
}

impl PleaseAck {
    pub fn contains(&self, ack_on: AckOn) -> bool {
        self.on.contains(&ack_on)
    }

    pub fn is_empty(&self) -> bool {
        self.on.is_empty()
    }

    pub fn from(ack_on: Vec<AckOn>) -> Self {
        Self { on: ack_on }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum AckOn {
    Receipt,
    Outcome,
}

#[macro_export]
macro_rules! please_ack (($type:ident) => (
    use crate::concepts::ack::please_ack::AckOn;
    impl $type {
        pub fn ask_for_ack(mut self) -> $type {
            self.please_ack = Some(PleaseAck::default());
            self
        }

        pub fn set_ack_on(mut self, ack_on: Vec<AckOn>) -> $type {
            self.please_ack = Some(PleaseAck::from(ack_on));
            self
        }

        pub fn ack_on(&self, ack_on: AckOn) -> bool {
            if let Some(please_ack) = &self.please_ack {
                please_ack.contains(ack_on)
            } else {
                false
            }
        }

        // Caution, some implementations assume Some(PleaseAck::default()) to be positive ack request!
        pub fn ack_on_any(&self) -> bool {
            if let Some(please_ack) = &self.please_ack {
                !please_ack.is_empty()
            } else {
                false
            }
        }
    }
));
