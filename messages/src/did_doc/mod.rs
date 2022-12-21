use url::Url;

use crate::concepts::aries_service::AriesService;
use w3c::model::{Authentication, CONTEXT, DdoKeyReference, Ed25519PublicKey, KEY_AUTHENTICATION_TYPE, KEY_TYPE};

use crate::utils::validation::validate_verkey;
use crate::errors::error::{MessagesError, MessagesErrorKind, MessagesResult};

pub mod service_didsov;
pub mod aries;
pub mod w3c;
