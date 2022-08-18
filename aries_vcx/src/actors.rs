use strum::IntoEnumIterator;

use crate::error::{VcxError, VcxErrorKind};
use crate::global::settings;
use crate::global::settings::CONFIG_ACTORS;

pub fn get_actors() -> Vec<Actors> {
    settings::get_config_value(CONFIG_ACTORS)
        .and_then(|actors| serde_json::from_str(&actors).map_err(|_| VcxError::from(VcxErrorKind::InvalidOption)))
        .unwrap_or_else(|_| Actors::iter().collect())
}

#[derive(Clone, Serialize, Deserialize, Debug, Eq, PartialEq, EnumIter)]
#[serde(rename_all = "lowercase")]
pub enum Actors {
    Inviter,
    Invitee,
    Issuer,
    Holder,
    Prover,
    Verifier,
    Sender,
    Receiver,
}
