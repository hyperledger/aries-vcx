use regex::Regex;

lazy_static! {
    pub static ref REGEX: Regex =
        Regex::new("did:([a-z0-9]+):([a-zA-Z0-9:.-_]*)").expect("Unexpected regex error occurred.");
}

pub fn is_fully_qualified(entity: &str) -> bool {
    REGEX.is_match(entity)
}

#[cfg(test)]
#[cfg(feature = "general_test")]
mod test {
    use super::*;

    #[test]
    fn is_fully_qualified_works() {
        assert!(is_fully_qualified("did:indy:some"));
        assert!(!is_fully_qualified("did:indy"));
        assert!(!is_fully_qualified("indy:some"));
    }
}
