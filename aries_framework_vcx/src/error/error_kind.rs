#[derive(Copy, Clone, Eq, PartialEq, Debug, thiserror::Error)]
pub enum VCXFrameworkErrorKind {
    #[error("VCXFramework error")]
    GenericVCXFrameworkError,
    #[error("Serialization error")]
    SerializationError,
}
