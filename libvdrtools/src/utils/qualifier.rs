use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    pub static ref REGEX: Regex =
        Regex::new("^[a-z0-9]+(:(indy|cheqd))?(:[a-z0-9:]+)?:(.*)$").unwrap();
}

pub fn qualify(entity: &str, prefix: &str, method: &str) -> String {
    format!("{}:{}:{}", prefix, method, entity)
}

pub fn qualify_with_ledger(entity: &str, prefix: &str, method: &str, ledger_type: &str) -> String {
    format!("{}:{}:{}:{}", prefix, method, ledger_type, entity)
}

pub fn to_unqualified(entity: &str) -> String {
    trace!("qualifier::to_unqualified >> {}", entity);
    match REGEX.captures(entity) {
        None => entity.to_string(),
        Some(caps) => {
            trace!("qualifier::to_unqualified: parts {:?}", caps);
            caps.get(4)
                .map(|m| m.as_str().to_string())
                .unwrap_or(entity.to_string())
        }
    }
}

pub fn method(entity: &str) -> Option<String> {
    match REGEX.captures(entity) {
        None => None,
        Some(caps) => {
            trace!("qualifier::method: caps {:?}", caps);
            match (caps.get(2), caps.get(3)) {
                (Some(type_), Some(subnet)) => Some(type_.as_str().to_owned() + subnet.as_str()),
                (Some(type_), None) => Some(type_.as_str().to_owned()),
                _ => {
                    warn!(
                        "Unrecognized FQ method for {}, parsed items are \
                    (where 2nd is method type, and 3rd is sub-method (namespace, ledger, type, etc)\
                     {:?}",
                        entity, caps
                    );
                    None
                }
            }
        }
    }
}

pub fn is_fully_qualified(entity: &str) -> bool {
    REGEX.is_match(&entity)
}

macro_rules! qualifiable_type (($newtype:ident) => (

    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
    pub struct $newtype(pub String);

    impl $newtype {

        #[allow(dead_code)]
        pub fn get_method(&self) -> Option<String> {
            qualifier::method(&self.0)
        }

        #[allow(dead_code)]
        pub fn set_method(&self, method: &str) -> $newtype {
            $newtype(qualifier::qualify(&self.0, $newtype::PREFIX, &method))
        }

        #[allow(dead_code)]
        pub fn set_ledger_and_method(&self, ledger_type: &str, method: &str) -> $newtype {
            $newtype(qualifier::qualify_with_ledger(&self.0, $newtype::PREFIX, method, ledger_type))
        }

        #[allow(dead_code)]
        pub fn is_fully_qualified(&self) -> bool {
            self.0.contains($newtype::PREFIX) && qualifier::is_fully_qualified(&self.0)
        }
    }

    impl From<&str> for $newtype {
        fn from(value: &str) -> Self {
            Self(value.to_owned())
        }
    }

    impl From<&String> for $newtype {
        fn from(value: &String) -> Self {
            Self(value.clone())
        }
    }
));
