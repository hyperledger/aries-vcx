#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct PleaseAck {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    on: Vec<AckOn>
}

impl PleaseAck {
    pub fn contains(&self, ack_on: AckOn) -> bool {
        self.on.contains(&ack_on)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum AckOn {
    Receipt,
    Outcome
}

#[macro_export]
macro_rules! please_ack (($type:ident) => (
    use crate::ack::please_ack::AckOn;
    impl $type {
        pub fn ask_for_ack(mut self) -> $type {
            self.please_ack = Some(PleaseAck::default());
            self
        }

        pub fn ack_on(&self, ack_on: AckOn) -> bool {
            if let Some(please_ack) = &self.please_ack {
                please_ack.contains(ack_on)
            } else {
                false
            }
        }
    }
));
