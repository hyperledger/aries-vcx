use crate::error::VCXFrameworkErrorKind;

#[derive(Debug)]
pub struct VCXFrameworkError {
    pub message: String,
    pub kind: VCXFrameworkErrorKind,
}

impl std::fmt::Display for VCXFrameworkError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        f.write_str(&format!("{}: {}", self.kind, self.message))
    }
}

impl VCXFrameworkError {
    pub fn from_msg(kind: VCXFrameworkErrorKind, msg: &str) -> Self {
        VCXFrameworkError {
            kind,
            message: msg.to_string(),
        }
    }

    pub fn from_kind(kind: VCXFrameworkErrorKind) -> Self {
        let message = kind.to_string();
        VCXFrameworkError { kind, message }
    }
}
