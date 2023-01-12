use indy_api_types::validation::Validatable;
use std::collections::HashSet;
use std::iter::IntoIterator;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Namespaces(pub HashSet<String>);

impl Into<HashSet<String>> for Namespaces {
    fn into(self) -> HashSet<String> {
        self.0
    }
}

impl Validatable for Namespaces {
    fn validate(&self) -> Result<(), String> {
        if self.0.is_empty() {
            return Err(String::from("Empty list of Namespaces has been passed"));
        }

        Ok(())
    }
}

impl IntoIterator for Namespaces {
    type Item = String;
    type IntoIter = std::collections::hash_set::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
