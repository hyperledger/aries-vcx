use std::string;

use serde::{
    de,
    ser::{Serialize, Serializer},
    Deserialize, Deserializer,
};
use serde_json::{self, json, Value as JsonValue};

/// An abstract query representation over a key and value type
#[derive(Debug, Hash, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum AbstractQuery<K, V> {
    /// Logical AND of multiple clauses
    And(Vec<Self>),
    /// Logical OR of multiple clauses
    Or(Vec<Self>),
    /// Negation of a clause
    Not(Box<Self>),
    /// Equality comparison for a field value
    Eq(K, V),
    /// Inequality comparison for a field value
    Neq(K, V),
    /// Greater-than comparison for a field value
    Gt(K, V),
    /// Greater-than-or-equal comparison for a field value
    Gte(K, V),
    /// Less-than comparison for a field value
    Lt(K, V),
    /// Less-than-or-equal comparison for a field value
    Lte(K, V),
    /// SQL 'LIKE'-compatible string comparison for a field value
    Like(K, V),
    /// Match one of multiple field values in a set
    In(K, Vec<V>),
    /// Match any non-null field value of the given field names
    Exist(Vec<K>),
}

impl<K, V> Default for AbstractQuery<K, V> {
    fn default() -> Self {
        Self::And(Vec::new())
    }
}

/// A concrete query implementation with String keys and values
pub type Query = AbstractQuery<String, String>;

impl<K, V> AbstractQuery<K, V> {
    /// Perform basic query clause optimization
    pub fn optimise(self) -> Option<Self> {
        match self {
            Self::Not(boxed_query) => match boxed_query.optimise() {
                None => None,
                Some(Self::Not(nested_query)) => Some(*nested_query),
                Some(other) => Some(Self::Not(Box::new(other))),
            },
            Self::And(subqueries) => {
                let mut subqueries: Vec<Self> =
                    subqueries.into_iter().filter_map(Self::optimise).collect();

                match subqueries.len() {
                    0 => None,
                    1 => Some(subqueries.remove(0)),
                    _ => Some(Self::And(subqueries)),
                }
            }
            Self::Or(subqueries) => {
                let mut subqueries: Vec<Self> =
                    subqueries.into_iter().filter_map(Self::optimise).collect();

                match subqueries.len() {
                    0 => None,
                    1 => Some(subqueries.remove(0)),
                    _ => Some(Self::Or(subqueries)),
                }
            }
            Self::In(key, mut targets) if targets.len() == 1 => {
                Some(Self::Eq(key, targets.remove(0)))
            }
            other => Some(other),
        }
    }

    pub fn get_name(&self) -> Vec<&K> {
        match self {
            Self::And(subqueries) | Self::Or(subqueries) => {
                subqueries.iter().flat_map(Self::get_name).collect()
            }
            Self::Exist(subquery_names) => subquery_names
                .to_owned()
                .iter()
                .map(|s| s.to_owned())
                .collect(),
            Self::Not(boxed_query) => boxed_query.get_name(),
            Self::Eq(tag_name, _)
            | Self::Neq(tag_name, _)
            | Self::Gt(tag_name, _)
            | Self::Gte(tag_name, _)
            | Self::Lt(tag_name, _)
            | Self::Lte(tag_name, _)
            | Self::Like(tag_name, _)
            | Self::In(tag_name, _) => vec![tag_name],
        }
    }

    /// Perform a transformation on all field names in query clauses
    pub fn map_names<RK, E>(
        self,
        mut f: impl FnMut(K) -> Result<RK, E>,
    ) -> Result<AbstractQuery<RK, V>, E> {
        self.map(&mut f, &mut |_k, v| Ok(v))
    }

    /// Perform a transformation on all field values in query clauses
    pub fn map_values<RV, E>(
        self,
        mut f: impl FnMut(&K, V) -> Result<RV, E>,
    ) -> Result<AbstractQuery<K, RV>, E> {
        self.map(&mut |k| Ok(k), &mut f)
    }

    /// Transform all query clauses using field name and value conversions
    pub fn map<RK, RV, KF, VF, E>(
        self,
        kf: &mut KF,
        vf: &mut VF,
    ) -> Result<AbstractQuery<RK, RV>, E>
    where
        KF: FnMut(K) -> Result<RK, E>,
        VF: FnMut(&K, V) -> Result<RV, E>,
    {
        match self {
            Self::Eq(tag_name, tag_value) => {
                let tag_value = vf(&tag_name, tag_value)?;
                Ok(AbstractQuery::<RK, RV>::Eq(kf(tag_name)?, tag_value))
            }
            Self::Neq(tag_name, tag_value) => {
                let tag_value = vf(&tag_name, tag_value)?;
                Ok(AbstractQuery::<RK, RV>::Neq(kf(tag_name)?, tag_value))
            }
            Self::Gt(tag_name, tag_value) => {
                let tag_value = vf(&tag_name, tag_value)?;
                Ok(AbstractQuery::<RK, RV>::Gt(kf(tag_name)?, tag_value))
            }
            Self::Gte(tag_name, tag_value) => {
                let tag_value = vf(&tag_name, tag_value)?;
                Ok(AbstractQuery::<RK, RV>::Gte(kf(tag_name)?, tag_value))
            }
            Self::Lt(tag_name, tag_value) => {
                let tag_value = vf(&tag_name, tag_value)?;
                Ok(AbstractQuery::<RK, RV>::Lt(kf(tag_name)?, tag_value))
            }
            Self::Lte(tag_name, tag_value) => {
                let tag_value = vf(&tag_name, tag_value)?;
                Ok(AbstractQuery::<RK, RV>::Lte(kf(tag_name)?, tag_value))
            }
            Self::Like(tag_name, tag_value) => {
                let tag_value = vf(&tag_name, tag_value)?;
                Ok(AbstractQuery::<RK, RV>::Like(kf(tag_name)?, tag_value))
            }
            Self::In(tag_name, tag_values) => {
                let tag_values = tag_values
                    .into_iter()
                    .map(|value| vf(&tag_name, value))
                    .collect::<Result<Vec<_>, E>>()?;
                Ok(AbstractQuery::<RK, RV>::In(kf(tag_name)?, tag_values))
            }
            Self::Exist(tag_names) => Ok(AbstractQuery::<RK, RV>::Exist(
                tag_names.into_iter().try_fold(vec![], |mut v, tag_name| {
                    v.push(kf(tag_name)?);
                    Result::<_, E>::Ok(v)
                })?,
            )),
            Self::And(subqueries) => {
                let subqueries = subqueries
                    .into_iter()
                    .map(|query| query.map(kf, vf))
                    .collect::<Result<Vec<_>, E>>()?;
                Ok(AbstractQuery::<RK, RV>::And(subqueries))
            }
            Self::Or(subqueries) => {
                let subqueries = subqueries
                    .into_iter()
                    .map(|query| query.map(kf, vf))
                    .collect::<Result<Vec<_>, E>>()?;
                Ok(AbstractQuery::<RK, RV>::Or(subqueries))
            }
            Self::Not(boxed_query) => Ok(AbstractQuery::<RK, RV>::Not(Box::new(
                boxed_query.map(kf, vf)?,
            ))),
        }
    }
}

impl<K, V> Serialize for AbstractQuery<K, V>
where
    for<'a> &'a K: Into<String>,
    V: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.to_value().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Query {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v = JsonValue::deserialize(deserializer)?;

        match v {
            JsonValue::Object(map) => parse_query(map).map_err(de::Error::missing_field),
            JsonValue::Array(array) => {
                // cast old restrictions format to wql
                let mut res: Vec<JsonValue> = Vec::new();
                for sub_query in array {
                    let sub_query: serde_json::Map<String, JsonValue> = sub_query
                        .as_object()
                        .ok_or_else(|| de::Error::custom("Restriction is invalid"))?
                        .clone()
                        .into_iter()
                        .filter(|(_, v)| !v.is_null())
                        .collect();

                    if !sub_query.is_empty() {
                        res.push(JsonValue::Object(sub_query));
                    }
                }

                let mut map = serde_json::Map::new();
                map.insert("$or".to_string(), JsonValue::Array(res));

                parse_query(map).map_err(de::Error::custom)
            }
            _ => Err(de::Error::missing_field(
                "Restriction must be either object or array",
            )),
        }
    }
}

impl<K, V> AbstractQuery<K, V>
where
    for<'a> &'a K: Into<String>,
    V: Serialize,
{
    fn to_value(&self) -> JsonValue {
        match self {
            Self::Eq(ref tag_name, ref tag_value) => json!({ tag_name: tag_value }),
            Self::Neq(ref tag_name, ref tag_value) => json!({tag_name: {"$neq": tag_value}}),
            Self::Gt(ref tag_name, ref tag_value) => json!({tag_name: {"$gt": tag_value}}),
            Self::Gte(ref tag_name, ref tag_value) => json!({tag_name: {"$gte": tag_value}}),
            Self::Lt(ref tag_name, ref tag_value) => json!({tag_name: {"$lt": tag_value}}),
            Self::Lte(ref tag_name, ref tag_value) => json!({tag_name: {"$lte": tag_value}}),
            Self::Like(ref tag_name, ref tag_value) => json!({tag_name: {"$like": tag_value}}),
            Self::In(ref tag_name, ref tag_values) => json!({tag_name: {"$in":tag_values}}),
            Self::Exist(ref tag_names) => {
                json!({ "$exist": tag_names.iter().map(Into::into).collect::<Vec<String>>() })
            }
            Self::And(ref queries) => {
                if queries.is_empty() {
                    json!({})
                } else {
                    json!({
                        "$and": queries.iter().map(Self::to_value).collect::<Vec<JsonValue>>()
                    })
                }
            }
            Self::Or(ref queries) => {
                if queries.is_empty() {
                    json!({})
                } else {
                    json!({
                        "$or": queries.iter().map(Self::to_value).collect::<Vec<JsonValue>>()
                    })
                }
            }
            Self::Not(ref query) => json!({"$not": query.to_value()}),
        }
    }
}

#[allow(clippy::to_string_trait_impl)] // mimicks upstream anoncreds-rs, allow this to avoid divergence
impl string::ToString for Query {
    fn to_string(&self) -> String {
        self.to_value().to_string()
    }
}

fn parse_query(map: serde_json::Map<String, JsonValue>) -> Result<Query, &'static str> {
    let mut operators: Vec<Query> = Vec::new();

    for (key, value) in map {
        if let Some(operator_) = parse_operator(key, value)? {
            operators.push(operator_);
        }
    }

    let query = if operators.len() == 1 {
        operators.remove(0)
    } else {
        Query::And(operators)
    };

    Ok(query)
}

fn parse_operator(key: String, value: JsonValue) -> Result<Option<Query>, &'static str> {
    match (key.as_str(), value) {
        ("$and", JsonValue::Array(values)) => {
            if values.is_empty() {
                Ok(None)
            } else {
                let operators: Vec<Query> = parse_list_operators(values)?;
                Ok(Some(Query::And(operators)))
            }
        }
        ("$and", _) => Err("$and must be array of JSON objects"),
        ("$or", JsonValue::Array(values)) => {
            if values.is_empty() {
                Ok(None)
            } else {
                let operators: Vec<Query> = parse_list_operators(values)?;
                Ok(Some(Query::Or(operators)))
            }
        }
        ("$or", _) => Err("$or must be array of JSON objects"),
        ("$not", JsonValue::Object(map)) => {
            let operator = parse_query(map)?;
            Ok(Some(Query::Not(Box::new(operator))))
        }
        ("$not", _) => Err("$not must be JSON object"),
        ("$exist", JsonValue::String(key)) => Ok(Some(Query::Exist(vec![key]))),
        ("$exist", JsonValue::Array(keys)) => {
            if keys.is_empty() {
                Ok(None)
            } else {
                let mut ks = vec![];
                for key in keys {
                    if let JsonValue::String(key) = key {
                        ks.push(key);
                    } else {
                        return Err("$exist must be used with a string or array of strings");
                    }
                }
                Ok(Some(Query::Exist(ks)))
            }
        }
        ("$exist", _) => Err("$exist must be used with a string or array of strings"),
        (_, JsonValue::String(value)) => Ok(Some(Query::Eq(key, value))),
        (_, JsonValue::Object(map)) => {
            if map.len() == 1 {
                let (operator_name, value) = map.into_iter().next().unwrap();
                parse_single_operator(operator_name.as_str(), key, value).map(Some)
            } else {
                Err("value must be JSON object of length 1")
            }
        }
        (_, _) => Err("Unsupported value"),
    }
}

fn parse_list_operators(operators: Vec<JsonValue>) -> Result<Vec<Query>, &'static str> {
    let mut out_operators: Vec<Query> = Vec::with_capacity(operators.len());

    for value in operators {
        if let JsonValue::Object(map) = value {
            let subquery = parse_query(map)?;
            out_operators.push(subquery);
        } else {
            return Err("operator must be array of JSON objects");
        }
    }

    Ok(out_operators)
}

fn parse_single_operator(
    operator_name: &str,
    key: String,
    value: JsonValue,
) -> Result<Query, &'static str> {
    match (operator_name, value) {
        ("$neq", JsonValue::String(value_)) => Ok(Query::Neq(key, value_)),
        ("$neq", _) => Err("$neq must be used with string"),
        ("$gt", JsonValue::String(value_)) => Ok(Query::Gt(key, value_)),
        ("$gt", _) => Err("$gt must be used with string"),
        ("$gte", JsonValue::String(value_)) => Ok(Query::Gte(key, value_)),
        ("$gte", _) => Err("$gte must be used with string"),
        ("$lt", JsonValue::String(value_)) => Ok(Query::Lt(key, value_)),
        ("$lt", _) => Err("$lt must be used with string"),
        ("$lte", JsonValue::String(value_)) => Ok(Query::Lte(key, value_)),
        ("$lte", _) => Err("$lte must be used with string"),
        ("$like", JsonValue::String(value_)) => Ok(Query::Like(key, value_)),
        ("$like", _) => Err("$like must be used with string"),
        ("$in", JsonValue::Array(values)) => {
            let mut target_values: Vec<String> = Vec::with_capacity(values.len());

            for v in values {
                if let JsonValue::String(s) = v {
                    target_values.push(s);
                } else {
                    return Err("$in must be used with array of strings");
                }
            }

            Ok(Query::In(key, target_values))
        }
        ("$in", _) => Err("$in must be used with array of strings"),
        (_, _) => Err("Unknown operator"),
    }
}

#[cfg(test)]
mod tests {
    use rand::{distributions::Alphanumeric, thread_rng, Rng};
    use serde_json::json;

    use super::*;

    fn _random_string(len: usize) -> String {
        String::from_utf8(thread_rng().sample_iter(&Alphanumeric).take(len).collect()).unwrap()
    }

    /// parse
    #[test]
    fn test_simple_operator_empty_json_parse() {
        let json = "{}";

        let query: Query = ::serde_json::from_str(json).unwrap();

        let expected = Query::And(vec![]);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_simple_operator_explicit_empty_and_parse() {
        let json = r#"{"$and":[]}"#;

        let query: Query = ::serde_json::from_str(json).unwrap();

        let expected = Query::And(vec![]);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_simple_operator_empty_or_parse() {
        let json = r#"{"$or":[]}"#;

        let query: Query = ::serde_json::from_str(json).unwrap();

        let expected = Query::And(vec![]);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_simple_operator_empty_not_parse() {
        let json = r#"{"$not":{}}"#;

        let query: Query = ::serde_json::from_str(json).unwrap();

        let expected = Query::Not(Box::new(Query::And(vec![])));

        assert_eq!(query, expected);
    }

    #[test]
    fn test_simple_operator_eq_plaintext_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let json = format!(r#"{{"{}":"{}"}}"#, name1, value1);

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::Eq(name1, value1);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_simple_operator_neq_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let json = format!(r#"{{"{}":{{"$neq":"{}"}}}}"#, name1, value1);

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::Neq(name1, value1);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_simple_operator_gt_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let json = format!(r#"{{"{}":{{"$gt":"{}"}}}}"#, name1, value1);

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::Gt(name1, value1);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_simple_operator_gte_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let json = format!(r#"{{"{}":{{"$gte":"{}"}}}}"#, name1, value1);

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::Gte(name1, value1);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_simple_operator_lt_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let json = format!(r#"{{"{}":{{"$lt":"{}"}}}}"#, name1, value1);

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::Lt(name1, value1);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_simple_operator_lte_plaintext_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let json = format!(r#"{{"{}":{{"$lte":"{}"}}}}"#, name1, value1);

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::Lte(name1, value1);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_simple_operator_like_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let json = format!(r#"{{"{}":{{"$like":"{}"}}}}"#, name1, value1);

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::Like(name1, value1);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_simple_operator_in_plaintext_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let json = format!(r#"{{"{}":{{"$in":["{}"]}}}}"#, name1, value1);

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::In(name1, vec![value1]);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_simple_operator_in_plaintexts_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let value2 = _random_string(10);
        let value3 = _random_string(10);

        let json = format!(
            r#"{{"{}":{{"$in":["{}","{}","{}"]}}}}"#,
            name1, value1, value2, value3
        );

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::In(name1, vec![value1, value2, value3]);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_exist_parse_string() {
        let name1 = _random_string(10);

        let json = format!(r#"{{"$exist":"{}"}}"#, name1);

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::Exist(vec![name1]);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_exist_parse_array() {
        let name1 = _random_string(10);
        let name2 = _random_string(10);

        let json = format!(r#"{{"$exist":["{}","{}"]}}"#, name1, name2);

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::Exist(vec![name1, name2]);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_and_exist() {
        let name1 = _random_string(10);
        let name2 = _random_string(10);

        let json = format!(
            r#"{{"$and":[{{"$exist":"{}"}},{{"$exist":"{}"}}]}}"#,
            name1, name2
        );

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::And(vec![Query::Exist(vec![name1]), Query::Exist(vec![name2])]);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_and_with_one_eq_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let json = format!(r#"{{"$and":[{{"{}":"{}"}}]}}"#, name1, value1);

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::And(vec![Query::Eq(name1, value1)]);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_and_with_one_neq_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let json = format!(r#"{{"$and":[{{"{}":{{"$neq":"{}"}}}}]}}"#, name1, value1);

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::And(vec![Query::Neq(name1, value1)]);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_and_with_one_gt_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let json = format!(r#"{{"$and":[{{"{}":{{"$gt":"{}"}}}}]}}"#, name1, value1);

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::And(vec![Query::Gt(name1, value1)]);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_and_with_one_gte_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let json = format!(r#"{{"$and":[{{"{}":{{"$gte":"{}"}}}}]}}"#, name1, value1);

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::And(vec![Query::Gte(name1, value1)]);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_and_with_one_lt_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let json = format!(r#"{{"$and":[{{"{}":{{"$lt":"{}"}}}}]}}"#, name1, value1);

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::And(vec![Query::Lt(name1, value1)]);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_and_with_one_lte_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let json = format!(r#"{{"$and":[{{"{}":{{"$lte":"{}"}}}}]}}"#, name1, value1);

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::And(vec![Query::Lte(name1, value1)]);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_and_with_one_like_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let json = format!(r#"{{"$and":[{{"{}":{{"$like":"{}"}}}}]}}"#, name1, value1);

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::And(vec![Query::Like(name1, value1)]);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_and_with_one_in_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let json = format!(r#"{{"$and":[{{"{}":{{"$in":["{}"]}}}}]}}"#, name1, value1);

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::And(vec![Query::In(name1, vec![value1])]);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_and_with_one_not_eq_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let json = format!(r#"{{"$and":[{{"$not":{{"{}":"{}"}}}}]}}"#, name1, value1);

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::And(vec![Query::Not(Box::new(Query::Eq(name1, value1)))]);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_short_and_with_multiple_eq_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);

        let json = format!(
            r#"{{"{}":"{}","{}":"{}","{}":"{}"}}"#,
            name1, value1, name2, value2, name3, value3,
        );

        let query: Query = ::serde_json::from_str(&json).unwrap();
        let mut clauses = vec![
            Query::Eq(name1, value1),
            Query::Eq(name2, value2),
            Query::Eq(name3, value3),
        ];
        clauses.sort();

        let expected = Query::And(clauses);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_and_with_multiple_eq_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);

        let json = format!(
            r#"{{"$and":[{{"{}":"{}"}},{{"{}":"{}"}},{{"{}":"{}"}}]}}"#,
            name1, value1, name2, value2, name3, value3,
        );

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::And(vec![
            Query::Eq(name1, value1),
            Query::Eq(name2, value2),
            Query::Eq(name3, value3),
        ]);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_and_with_multiple_neq_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);

        let json = format!(
            r#"{{"$and":[{{"{}":{{"$neq":"{}"}}}},{{"{}":{{"$neq":"{}"}}}},{{"{}":{{"$neq":"{}"}}}}]}}"#,
            name1, value1, name2, value2, name3, value3,
        );

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::And(vec![
            Query::Neq(name1, value1),
            Query::Neq(name2, value2),
            Query::Neq(name3, value3),
        ]);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_and_with_multiple_gt_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);

        let json = format!(
            r#"{{"$and":[{{"{}":{{"$gt":"{}"}}}},{{"{}":{{"$gt":"{}"}}}},{{"{}":{{"$gt":"{}"}}}}]}}"#,
            name1, value1, name2, value2, name3, value3,
        );

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::And(vec![
            Query::Gt(name1, value1),
            Query::Gt(name2, value2),
            Query::Gt(name3, value3),
        ]);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_and_with_multiple_gte_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);

        let json = format!(
            r#"{{"$and":[{{"{}":{{"$gte":"{}"}}}},{{"{}":{{"$gte":"{}"}}}},{{"{}":{{"$gte":"{}"}}}}]}}"#,
            name1, value1, name2, value2, name3, value3,
        );

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::And(vec![
            Query::Gte(name1, value1),
            Query::Gte(name2, value2),
            Query::Gte(name3, value3),
        ]);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_and_with_multiple_lt_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);

        let json = format!(
            r#"{{"$and":[{{"{}":{{"$lt":"{}"}}}},{{"{}":{{"$lt":"{}"}}}},{{"{}":{{"$lt":"{}"}}}}]}}"#,
            name1, value1, name2, value2, name3, value3,
        );

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::And(vec![
            Query::Lt(name1, value1),
            Query::Lt(name2, value2),
            Query::Lt(name3, value3),
        ]);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_and_with_multiple_lte_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);

        let json = format!(
            r#"{{"$and":[{{"{}":{{"$lte":"{}"}}}},{{"{}":{{"$lte":"{}"}}}},{{"{}":{{"$lte":"{}"}}}}]}}"#,
            name1, value1, name2, value2, name3, value3,
        );

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::And(vec![
            Query::Lte(name1, value1),
            Query::Lte(name2, value2),
            Query::Lte(name3, value3),
        ]);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_and_with_multiple_like_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);

        let json = format!(
            r#"{{"$and":[{{"{}":{{"$like":"{}"}}}},{{"{}":{{"$like":"{}"}}}},{{"{}":{{"$like":"{}"}}}}]}}"#,
            name1, value1, name2, value2, name3, value3,
        );

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::And(vec![
            Query::Like(name1, value1),
            Query::Like(name2, value2),
            Query::Like(name3, value3),
        ]);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_and_with_multiple_in_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);

        let json = format!(
            r#"{{"$and":[{{"{}":{{"$in":["{}"]}}}},{{"{}":{{"$in":["{}"]}}}},{{"{}":{{"$in":["{}"]}}}}]}}"#,
            name1, value1, name2, value2, name3, value3,
        );

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::And(vec![
            Query::In(name1, vec![value1]),
            Query::In(name2, vec![value2]),
            Query::In(name3, vec![value3]),
        ]);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_and_with_multiple_not_eq_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);

        let json = format!(
            r#"{{"$and":[{{"$not":{{"{}":"{}"}}}},{{"$not":{{"{}":"{}"}}}},{{"$not":{{"{}":"{}"}}}}]}}"#,
            name1, value1, name2, value2, name3, value3,
        );

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::And(vec![
            Query::Not(Box::new(Query::Eq(name1, value1))),
            Query::Not(Box::new(Query::Eq(name2, value2))),
            Query::Not(Box::new(Query::Eq(name3, value3))),
        ]);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_and_with_multiple_mixed_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);
        let name4 = _random_string(10);
        let value4 = _random_string(10);
        let name5 = _random_string(10);
        let value5 = _random_string(10);
        let name6 = _random_string(10);
        let value6 = _random_string(10);
        let name7 = _random_string(10);
        let value7 = _random_string(10);
        let name8 = _random_string(10);
        let value8a = _random_string(10);
        let value8b = _random_string(10);
        let name9 = _random_string(10);
        let value9 = _random_string(10);

        let json = format!(
            r#"{{"$and":[{{"{}":"{}"}},{{"{}":{{"$neq":"{}"}}}},{{"{}":{{"$gt":"{}"}}}},{{"{}":{{"$gte":"{}"}}}},{{"{}":{{"$lt":"{}"}}}},{{"{}":{{"$lte":"{}"}}}},{{"{}":{{"$like":"{}"}}}},{{"{}":{{"$in":["{}","{}"]}}}},{{"$not":{{"{}":"{}"}}}}]}}"#,
            name1,
            value1,
            name2,
            value2,
            name3,
            value3,
            name4,
            value4,
            name5,
            value5,
            name6,
            value6,
            name7,
            value7,
            name8,
            value8a,
            value8b,
            name9,
            value9,
        );

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::And(vec![
            Query::Eq(name1, value1),
            Query::Neq(name2, value2),
            Query::Gt(name3, value3),
            Query::Gte(name4, value4),
            Query::Lt(name5, value5),
            Query::Lte(name6, value6),
            Query::Like(name7, value7),
            Query::In(name8, vec![value8a, value8b]),
            Query::Not(Box::new(Query::Eq(name9, value9))),
        ]);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_or_with_one_eq_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let json = format!(r#"{{"$or":[{{"{}":"{}"}}]}}"#, name1, value1);

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::Or(vec![Query::Eq(name1, value1)]);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_or_with_one_neq_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let json = format!(r#"{{"$or":[{{"{}":{{"$neq":"{}"}}}}]}}"#, name1, value1);

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::Or(vec![Query::Neq(name1, value1)]);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_or_with_one_gt_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let json = format!(r#"{{"$or":[{{"{}":{{"$gt":"{}"}}}}]}}"#, name1, value1);

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::Or(vec![Query::Gt(name1, value1)]);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_or_with_one_gte_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let json = format!(r#"{{"$or":[{{"{}":{{"$gte":"{}"}}}}]}}"#, name1, value1);

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::Or(vec![Query::Gte(name1, value1)]);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_or_with_one_lt_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let json = format!(r#"{{"$or":[{{"{}":{{"$lt":"{}"}}}}]}}"#, name1, value1);

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::Or(vec![Query::Lt(name1, value1)]);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_or_with_one_lte_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let json = format!(r#"{{"$or":[{{"{}":{{"$lte":"{}"}}}}]}}"#, name1, value1);

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::Or(vec![Query::Lte(name1, value1)]);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_or_with_one_like_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let json = format!(r#"{{"$or":[{{"{}":{{"$like":"{}"}}}}]}}"#, name1, value1);

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::Or(vec![Query::Like(name1, value1)]);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_or_with_one_in_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let json = format!(r#"{{"$or":[{{"{}":{{"$in":["{}"]}}}}]}}"#, name1, value1);

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::Or(vec![Query::In(name1, vec![value1])]);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_or_with_one_not_eq_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let json = format!(r#"{{"$or":[{{"$not":{{"{}":"{}"}}}}]}}"#, name1, value1);

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::Or(vec![Query::Not(Box::new(Query::Eq(name1, value1)))]);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_or_with_multiple_eq_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);

        let json = format!(
            r#"{{"$or":[{{"{}":"{}"}},{{"{}":"{}"}},{{"{}":"{}"}}]}}"#,
            name1, value1, name2, value2, name3, value3,
        );

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::Or(vec![
            Query::Eq(name1, value1),
            Query::Eq(name2, value2),
            Query::Eq(name3, value3),
        ]);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_or_with_multiple_neq_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);

        let json = format!(
            r#"{{"$or":[{{"{}":{{"$neq":"{}"}}}},{{"{}":{{"$neq":"{}"}}}},{{"{}":{{"$neq":"{}"}}}}]}}"#,
            name1, value1, name2, value2, name3, value3,
        );

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::Or(vec![
            Query::Neq(name1, value1),
            Query::Neq(name2, value2),
            Query::Neq(name3, value3),
        ]);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_or_with_multiple_gt_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);

        let json = format!(
            r#"{{"$or":[{{"{}":{{"$gt":"{}"}}}},{{"{}":{{"$gt":"{}"}}}},{{"{}":{{"$gt":"{}"}}}}]}}"#,
            name1, value1, name2, value2, name3, value3,
        );

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::Or(vec![
            Query::Gt(name1, value1),
            Query::Gt(name2, value2),
            Query::Gt(name3, value3),
        ]);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_or_with_multiple_gte_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);

        let json = format!(
            r#"{{"$or":[{{"{}":{{"$gte":"{}"}}}},{{"{}":{{"$gte":"{}"}}}},{{"{}":{{"$gte":"{}"}}}}]}}"#,
            name1, value1, name2, value2, name3, value3,
        );

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::Or(vec![
            Query::Gte(name1, value1),
            Query::Gte(name2, value2),
            Query::Gte(name3, value3),
        ]);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_or_with_multiple_lt_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);

        let json = format!(
            r#"{{"$or":[{{"{}":{{"$lt":"{}"}}}},{{"{}":{{"$lt":"{}"}}}},{{"{}":{{"$lt":"{}"}}}}]}}"#,
            name1, value1, name2, value2, name3, value3,
        );

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::Or(vec![
            Query::Lt(name1, value1),
            Query::Lt(name2, value2),
            Query::Lt(name3, value3),
        ]);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_or_with_multiple_lte_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);

        let json = format!(
            r#"{{"$or":[{{"{}":{{"$lte":"{}"}}}},{{"{}":{{"$lte":"{}"}}}},{{"{}":{{"$lte":"{}"}}}}]}}"#,
            name1, value1, name2, value2, name3, value3,
        );

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::Or(vec![
            Query::Lte(name1, value1),
            Query::Lte(name2, value2),
            Query::Lte(name3, value3),
        ]);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_or_with_multiple_like_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);

        let json = format!(
            r#"{{"$or":[{{"{}":{{"$like":"{}"}}}},{{"{}":{{"$like":"{}"}}}},{{"{}":{{"$like":"{}"}}}}]}}"#,
            name1, value1, name2, value2, name3, value3,
        );

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::Or(vec![
            Query::Like(name1, value1),
            Query::Like(name2, value2),
            Query::Like(name3, value3),
        ]);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_or_with_multiple_in_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);

        let json = format!(
            r#"{{"$or":[{{"{}":{{"$in":["{}"]}}}},{{"{}":{{"$in":["{}"]}}}},{{"{}":{{"$in":["{}"]}}}}]}}"#,
            name1, value1, name2, value2, name3, value3,
        );

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::Or(vec![
            Query::In(name1, vec![value1]),
            Query::In(name2, vec![value2]),
            Query::In(name3, vec![value3]),
        ]);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_or_with_multiple_not_eq_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);

        let json = format!(
            r#"{{"$or":[{{"$not":{{"{}":"{}"}}}},{{"$not":{{"{}":"{}"}}}},{{"$not":{{"{}":"{}"}}}}]}}"#,
            name1, value1, name2, value2, name3, value3,
        );

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::Or(vec![
            Query::Not(Box::new(Query::Eq(name1, value1))),
            Query::Not(Box::new(Query::Eq(name2, value2))),
            Query::Not(Box::new(Query::Eq(name3, value3))),
        ]);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_or_with_multiple_mixed_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);
        let name4 = _random_string(10);
        let value4 = _random_string(10);
        let name5 = _random_string(10);
        let value5 = _random_string(10);
        let name6 = _random_string(10);
        let value6 = _random_string(10);
        let name7 = _random_string(10);
        let value7 = _random_string(10);
        let name8 = _random_string(10);
        let value8a = _random_string(10);
        let value8b = _random_string(10);
        let name9 = _random_string(10);
        let value9 = _random_string(10);

        let json = format!(
            r#"{{"$or":[{{"{}":"{}"}},{{"{}":{{"$neq":"{}"}}}},{{"{}":{{"$gt":"{}"}}}},{{"{}":{{"$gte":"{}"}}}},{{"{}":{{"$lt":"{}"}}}},{{"{}":{{"$lte":"{}"}}}},{{"{}":{{"$like":"{}"}}}},{{"{}":{{"$in":["{}","{}"]}}}},{{"$not":{{"{}":"{}"}}}}]}}"#,
            name1,
            value1,
            name2,
            value2,
            name3,
            value3,
            name4,
            value4,
            name5,
            value5,
            name6,
            value6,
            name7,
            value7,
            name8,
            value8a,
            value8b,
            name9,
            value9,
        );

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::Or(vec![
            Query::Eq(name1, value1),
            Query::Neq(name2, value2),
            Query::Gt(name3, value3),
            Query::Gte(name4, value4),
            Query::Lt(name5, value5),
            Query::Lte(name6, value6),
            Query::Like(name7, value7),
            Query::In(name8, vec![value8a, value8b]),
            Query::Not(Box::new(Query::Eq(name9, value9))),
        ]);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_not_with_one_eq_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let json = format!(r#"{{"$not":{{"{}":"{}"}}}}"#, name1, value1);

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::Not(Box::new(Query::Eq(name1, value1)));

        assert_eq!(query, expected);
    }

    #[test]
    fn test_not_with_one_neq_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let json = format!(r#"{{"$not":{{"{}":{{"$neq":"{}"}}}}}}"#, name1, value1);

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::Not(Box::new(Query::Neq(name1, value1)));

        assert_eq!(query, expected);
    }

    #[test]
    fn test_not_with_one_gt_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let json = format!(r#"{{"$not":{{"{}":{{"$gt":"{}"}}}}}}"#, name1, value1);

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::Not(Box::new(Query::Gt(name1, value1)));

        assert_eq!(query, expected);
    }

    #[test]
    fn test_not_with_one_gte_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let json = format!(r#"{{"$not":{{"{}":{{"$gte":"{}"}}}}}}"#, name1, value1);

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::Not(Box::new(Query::Gte(name1, value1)));

        assert_eq!(query, expected);
    }

    #[test]
    fn test_not_with_one_lt_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let json = format!(r#"{{"$not":{{"{}":{{"$lt":"{}"}}}}}}"#, name1, value1);

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::Not(Box::new(Query::Lt(name1, value1)));

        assert_eq!(query, expected);
    }

    #[test]
    fn test_not_with_one_lte_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let json = format!(r#"{{"$not":{{"{}":{{"$lte":"{}"}}}}}}"#, name1, value1);

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::Not(Box::new(Query::Lte(name1, value1)));

        assert_eq!(query, expected);
    }

    #[test]
    fn test_not_with_one_like_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let json = format!(r#"{{"$not":{{"{}":{{"$like":"{}"}}}}}}"#, name1, value1);

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::Not(Box::new(Query::Like(name1, value1)));

        assert_eq!(query, expected);
    }

    #[test]
    fn test_not_with_one_in_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let json = format!(r#"{{"$not":{{"{}":{{"$in":["{}"]}}}}}}"#, name1, value1);

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::Not(Box::new(Query::In(name1, vec![value1])));

        assert_eq!(query, expected);
    }

    #[test]
    fn test_and_or_not_complex_case_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);
        let name4 = _random_string(10);
        let value4 = _random_string(10);
        let name5 = _random_string(10);
        let value5 = _random_string(10);
        let name6 = _random_string(10);
        let value6 = _random_string(10);
        let name7 = _random_string(10);
        let value7 = _random_string(10);
        let name8 = _random_string(10);
        let value8 = _random_string(10);

        let json = format!(
            r#"{{"$not":{{"$and":[{{"{}":"{}"}},{{"$or":[{{"{}":{{"$gt":"{}"}}}},{{"$not":{{"{}":{{"$lte":"{}"}}}}}},{{"$and":[{{"{}":{{"$lt":"{}"}}}},{{"$not":{{"{}":{{"$gte":"{}"}}}}}}]}}]}},{{"$not":{{"{}":{{"$like":"{}"}}}}}},{{"$and":[{{"{}":"{}"}},{{"$not":{{"{}":{{"$neq":"{}"}}}}}}]}}]}}}}"#,
            name1,
            value1,
            name2,
            value2,
            name3,
            value3,
            name4,
            value4,
            name5,
            value5,
            name6,
            value6,
            name7,
            value7,
            name8,
            value8,
        );

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::Not(Box::new(Query::And(vec![
            Query::Eq(name1, value1),
            Query::Or(vec![
                Query::Gt(name2, value2),
                Query::Not(Box::new(Query::Lte(name3, value3))),
                Query::And(vec![
                    Query::Lt(name4, value4),
                    Query::Not(Box::new(Query::Gte(name5, value5))),
                ]),
            ]),
            Query::Not(Box::new(Query::Like(name6, value6))),
            Query::And(vec![
                Query::Eq(name7, value7),
                Query::Not(Box::new(Query::Neq(name8, value8))),
            ]),
        ])));

        assert_eq!(query, expected);
    }

    /// to string
    #[test]
    fn test_simple_operator_empty_and_to_string() {
        let query = Query::And(vec![]);

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = "{}";

        assert_eq!(json, expected);
    }

    #[test]
    fn test_simple_operator_empty_or_to_string() {
        let query = Query::Or(vec![]);

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = "{}";

        assert_eq!(json, expected);
    }

    #[test]
    fn test_simple_operator_empty_not_to_string() {
        let query = Query::Not(Box::new(Query::And(vec![])));

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = r#"{"$not":{}}"#;

        assert_eq!(json, expected);
    }

    #[test]
    fn test_simple_operator_eq_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let query = Query::Eq(name1.clone(), value1.clone());

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"{}":"{}"}}"#, name1, value1);

        assert_eq!(json, expected);
    }

    #[test]
    fn test_simple_operator_neq_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let query = Query::Neq(name1.clone(), value1.clone());

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"{}":{{"$neq":"{}"}}}}"#, name1, value1);

        assert_eq!(json, expected);
    }

    #[test]
    fn test_simple_operator_gt_plaintext_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let query = Query::Gt(name1.clone(), value1.clone());

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"{}":{{"$gt":"{}"}}}}"#, name1, value1);

        assert_eq!(json, expected);
    }

    #[test]
    fn test_simple_operator_gte_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let query = Query::Gte(name1.clone(), value1.clone());

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"{}":{{"$gte":"{}"}}}}"#, name1, value1);

        assert_eq!(json, expected);
    }

    #[test]
    fn test_simple_operator_lt_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let query = Query::Lt(name1.clone(), value1.clone());

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"{}":{{"$lt":"{}"}}}}"#, name1, value1);

        assert_eq!(json, expected);
    }

    #[test]
    fn test_simple_operator_lte_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let query = Query::Lte(name1.clone(), value1.clone());

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"{}":{{"$lte":"{}"}}}}"#, name1, value1);

        assert_eq!(json, expected);
    }

    #[test]
    fn test_simple_operator_like_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let query = Query::Like(name1.clone(), value1.clone());

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"{}":{{"$like":"{}"}}}}"#, name1, value1);

        assert_eq!(json, expected);
    }

    #[test]
    fn test_simple_operator_in_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let query = Query::In(name1.clone(), vec![value1.clone()]);

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"{}":{{"$in":["{}"]}}}}"#, name1, value1);

        assert_eq!(json, expected);
    }

    #[test]
    fn test_simple_operator_in_multiply_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let value2 = _random_string(10);
        let value3 = _random_string(10);

        let query = Query::In(
            name1.clone(),
            vec![value1.clone(), value2.clone(), value3.clone()],
        );

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(
            r#"{{"{}":{{"$in":["{}","{}","{}"]}}}}"#,
            name1, value1, value2, value3
        );

        assert_eq!(json, expected);
    }

    #[test]
    fn test_and_with_one_eq_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let query = Query::And(vec![Query::Eq(name1.clone(), value1.clone())]);

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"$and":[{{"{}":"{}"}}]}}"#, name1, value1);

        assert_eq!(json, expected);
    }

    #[test]
    fn test_and_with_one_neq_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let query = Query::And(vec![Query::Neq(name1.clone(), value1.clone())]);

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"$and":[{{"{}":{{"$neq":"{}"}}}}]}}"#, name1, value1);

        assert_eq!(json, expected);
    }

    #[test]
    fn test_and_with_one_gt_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let query = Query::And(vec![Query::Gt(name1.clone(), value1.clone())]);

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"$and":[{{"{}":{{"$gt":"{}"}}}}]}}"#, name1, value1);

        assert_eq!(json, expected);
    }

    #[test]
    fn test_and_with_one_gte_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let query = Query::And(vec![Query::Gte(name1.clone(), value1.clone())]);

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"$and":[{{"{}":{{"$gte":"{}"}}}}]}}"#, name1, value1);

        assert_eq!(json, expected);
    }

    #[test]
    fn test_and_with_one_lt_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let query = Query::And(vec![Query::Lt(name1.clone(), value1.clone())]);

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"$and":[{{"{}":{{"$lt":"{}"}}}}]}}"#, name1, value1);

        assert_eq!(json, expected);
    }

    #[test]
    fn test_and_with_one_lte_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let query = Query::And(vec![Query::Lte(name1.clone(), value1.clone())]);

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"$and":[{{"{}":{{"$lte":"{}"}}}}]}}"#, name1, value1);

        assert_eq!(json, expected);
    }

    #[test]
    fn test_and_with_one_like_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let query = Query::And(vec![Query::Like(name1.clone(), value1.clone())]);

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"$and":[{{"{}":{{"$like":"{}"}}}}]}}"#, name1, value1);

        assert_eq!(json, expected);
    }

    #[test]
    fn test_and_with_one_in_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let query = Query::And(vec![Query::In(name1.clone(), vec![value1.clone()])]);

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"$and":[{{"{}":{{"$in":["{}"]}}}}]}}"#, name1, value1);

        assert_eq!(json, expected);
    }

    #[test]
    fn test_and_with_one_not_eq_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let query = Query::And(vec![Query::Not(Box::new(Query::Eq(
            name1.clone(),
            value1.clone(),
        )))]);

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"$and":[{{"$not":{{"{}":"{}"}}}}]}}"#, name1, value1);

        assert_eq!(json, expected);
    }

    #[test]
    fn test_and_with_multiple_eq_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);

        let query = Query::And(vec![
            Query::Eq(name1.clone(), value1.clone()),
            Query::Eq(name2.clone(), value2.clone()),
            Query::Eq(name3.clone(), value3.clone()),
        ]);

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(
            r#"{{"$and":[{{"{}":"{}"}},{{"{}":"{}"}},{{"{}":"{}"}}]}}"#,
            name1, value1, name2, value2, name3, value3,
        );

        assert_eq!(json, expected);
    }

    #[test]
    fn test_and_with_multiple_neq_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);

        let query = Query::And(vec![
            Query::Neq(name1.clone(), value1.clone()),
            Query::Neq(name2.clone(), value2.clone()),
            Query::Neq(name3.clone(), value3.clone()),
        ]);

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(
            r#"{{"$and":[{{"{}":{{"$neq":"{}"}}}},{{"{}":{{"$neq":"{}"}}}},{{"{}":{{"$neq":"{}"}}}}]}}"#,
            name1, value1, name2, value2, name3, value3,
        );

        assert_eq!(json, expected);
    }

    #[test]
    fn test_and_with_multiple_gt_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);

        let query = Query::And(vec![
            Query::Gt(name1.clone(), value1.clone()),
            Query::Gt(name2.clone(), value2.clone()),
            Query::Gt(name3.clone(), value3.clone()),
        ]);

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(
            r#"{{"$and":[{{"{}":{{"$gt":"{}"}}}},{{"{}":{{"$gt":"{}"}}}},{{"{}":{{"$gt":"{}"}}}}]}}"#,
            name1, value1, name2, value2, name3, value3,
        );

        assert_eq!(json, expected);
    }

    #[test]
    fn test_and_with_multiple_gte_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);

        let query = Query::And(vec![
            Query::Gte(name1.clone(), value1.clone()),
            Query::Gte(name2.clone(), value2.clone()),
            Query::Gte(name3.clone(), value3.clone()),
        ]);

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(
            r#"{{"$and":[{{"{}":{{"$gte":"{}"}}}},{{"{}":{{"$gte":"{}"}}}},{{"{}":{{"$gte":"{}"}}}}]}}"#,
            name1, value1, name2, value2, name3, value3,
        );

        assert_eq!(json, expected);
    }

    #[test]
    fn test_and_with_multiple_lt_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);

        let query = Query::And(vec![
            Query::Lt(name1.clone(), value1.clone()),
            Query::Lt(name2.clone(), value2.clone()),
            Query::Lt(name3.clone(), value3.clone()),
        ]);

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(
            r#"{{"$and":[{{"{}":{{"$lt":"{}"}}}},{{"{}":{{"$lt":"{}"}}}},{{"{}":{{"$lt":"{}"}}}}]}}"#,
            name1, value1, name2, value2, name3, value3,
        );

        assert_eq!(json, expected);
    }

    #[test]
    fn test_and_with_multiple_lte_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);

        let query = Query::And(vec![
            Query::Lte(name1.clone(), value1.clone()),
            Query::Lte(name2.clone(), value2.clone()),
            Query::Lte(name3.clone(), value3.clone()),
        ]);

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(
            r#"{{"$and":[{{"{}":{{"$lte":"{}"}}}},{{"{}":{{"$lte":"{}"}}}},{{"{}":{{"$lte":"{}"}}}}]}}"#,
            name1, value1, name2, value2, name3, value3,
        );

        assert_eq!(json, expected);
    }

    #[test]
    fn test_and_with_multiple_like_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);

        let query = Query::And(vec![
            Query::Like(name1.clone(), value1.clone()),
            Query::Like(name2.clone(), value2.clone()),
            Query::Like(name3.clone(), value3.clone()),
        ]);

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(
            r#"{{"$and":[{{"{}":{{"$like":"{}"}}}},{{"{}":{{"$like":"{}"}}}},{{"{}":{{"$like":"{}"}}}}]}}"#,
            name1, value1, name2, value2, name3, value3,
        );

        assert_eq!(json, expected);
    }

    #[test]
    fn test_and_with_multiple_in_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);

        let query = Query::And(vec![
            Query::In(name1.clone(), vec![value1.clone()]),
            Query::In(name2.clone(), vec![value2.clone()]),
            Query::In(name3.clone(), vec![value3.clone()]),
        ]);

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(
            r#"{{"$and":[{{"{}":{{"$in":["{}"]}}}},{{"{}":{{"$in":["{}"]}}}},{{"{}":{{"$in":["{}"]}}}}]}}"#,
            name1, value1, name2, value2, name3, value3,
        );

        assert_eq!(json, expected);
    }

    #[test]
    fn test_and_with_multiple_not_eq_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);

        let query = Query::And(vec![
            Query::Not(Box::new(Query::Eq(name1.clone(), value1.clone()))),
            Query::Not(Box::new(Query::Eq(name2.clone(), value2.clone()))),
            Query::Not(Box::new(Query::Eq(name3.clone(), value3.clone()))),
        ]);

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(
            r#"{{"$and":[{{"$not":{{"{}":"{}"}}}},{{"$not":{{"{}":"{}"}}}},{{"$not":{{"{}":"{}"}}}}]}}"#,
            name1, value1, name2, value2, name3, value3,
        );

        assert_eq!(json, expected);
    }

    #[test]
    fn test_and_with_multiple_mixed_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);
        let name4 = _random_string(10);
        let value4 = _random_string(10);
        let name5 = _random_string(10);
        let value5 = _random_string(10);
        let name6 = _random_string(10);
        let value6 = _random_string(10);
        let name7 = _random_string(10);
        let value7 = _random_string(10);
        let name8 = _random_string(10);
        let value8a = _random_string(10);
        let value8b = _random_string(10);
        let name9 = _random_string(10);
        let value9 = _random_string(10);

        let query = Query::And(vec![
            Query::Eq(name1.clone(), value1.clone()),
            Query::Neq(name2.clone(), value2.clone()),
            Query::Gt(name3.clone(), value3.clone()),
            Query::Gte(name4.clone(), value4.clone()),
            Query::Lt(name5.clone(), value5.clone()),
            Query::Lte(name6.clone(), value6.clone()),
            Query::Like(name7.clone(), value7.clone()),
            Query::In(name8.clone(), vec![value8a.clone(), value8b.clone()]),
            Query::Not(Box::new(Query::Eq(name9.clone(), value9.clone()))),
        ]);

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(
            r#"{{"$and":[{{"{}":"{}"}},{{"{}":{{"$neq":"{}"}}}},{{"{}":{{"$gt":"{}"}}}},{{"{}":{{"$gte":"{}"}}}},{{"{}":{{"$lt":"{}"}}}},{{"{}":{{"$lte":"{}"}}}},{{"{}":{{"$like":"{}"}}}},{{"{}":{{"$in":["{}","{}"]}}}},{{"$not":{{"{}":"{}"}}}}]}}"#,
            name1,
            value1,
            name2,
            value2,
            name3,
            value3,
            name4,
            value4,
            name5,
            value5,
            name6,
            value6,
            name7,
            value7,
            name8,
            value8a,
            value8b,
            name9,
            value9,
        );

        assert_eq!(json, expected);
    }

    #[test]
    fn test_or_with_one_eq_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let query = Query::Or(vec![Query::Eq(name1.clone(), value1.clone())]);

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"$or":[{{"{}":"{}"}}]}}"#, name1, value1);

        assert_eq!(json, expected);
    }

    #[test]
    fn test_or_with_one_neq_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let query = Query::Or(vec![Query::Neq(name1.clone(), value1.clone())]);

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"$or":[{{"{}":{{"$neq":"{}"}}}}]}}"#, name1, value1);

        assert_eq!(json, expected);
    }

    #[test]
    fn test_or_with_one_gt_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let query = Query::Or(vec![Query::Gt(name1.clone(), value1.clone())]);

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"$or":[{{"{}":{{"$gt":"{}"}}}}]}}"#, name1, value1);
        assert_eq!(json, expected);
    }

    #[test]
    fn test_or_with_one_gte_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let query = Query::Or(vec![Query::Gte(name1.clone(), value1.clone())]);

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"$or":[{{"{}":{{"$gte":"{}"}}}}]}}"#, name1, value1);

        assert_eq!(json, expected);
    }

    #[test]
    fn test_or_with_one_lt_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let query = Query::Or(vec![Query::Lt(name1.clone(), value1.clone())]);

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"$or":[{{"{}":{{"$lt":"{}"}}}}]}}"#, name1, value1);

        assert_eq!(json, expected);
    }

    #[test]
    fn test_or_with_one_lte_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let query = Query::Or(vec![Query::Lte(name1.clone(), value1.clone())]);

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"$or":[{{"{}":{{"$lte":"{}"}}}}]}}"#, name1, value1);

        assert_eq!(json, expected);
    }

    #[test]
    fn test_or_with_one_like_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let query = Query::Or(vec![Query::Like(name1.clone(), value1.clone())]);

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"$or":[{{"{}":{{"$like":"{}"}}}}]}}"#, name1, value1);

        assert_eq!(json, expected);
    }

    #[test]
    fn test_or_with_one_in_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let query = Query::Or(vec![Query::In(name1.clone(), vec![value1.clone()])]);

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"$or":[{{"{}":{{"$in":["{}"]}}}}]}}"#, name1, value1);

        assert_eq!(json, expected);
    }

    #[test]
    fn test_or_with_one_not_eq_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let query = Query::Or(vec![Query::Not(Box::new(Query::Eq(
            name1.clone(),
            value1.clone(),
        )))]);

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"$or":[{{"$not":{{"{}":"{}"}}}}]}}"#, name1, value1);

        assert_eq!(json, expected);
    }

    #[test]
    fn test_or_with_multiple_eq_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);

        let query = Query::Or(vec![
            Query::Eq(name1.clone(), value1.clone()),
            Query::Eq(name2.clone(), value2.clone()),
            Query::Eq(name3.clone(), value3.clone()),
        ]);

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(
            r#"{{"$or":[{{"{}":"{}"}},{{"{}":"{}"}},{{"{}":"{}"}}]}}"#,
            name1, value1, name2, value2, name3, value3,
        );

        assert_eq!(json, expected);
    }

    #[test]
    fn test_or_with_multiple_neq_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);

        let query = Query::Or(vec![
            Query::Neq(name1.clone(), value1.clone()),
            Query::Neq(name2.clone(), value2.clone()),
            Query::Neq(name3.clone(), value3.clone()),
        ]);

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(
            r#"{{"$or":[{{"{}":{{"$neq":"{}"}}}},{{"{}":{{"$neq":"{}"}}}},{{"{}":{{"$neq":"{}"}}}}]}}"#,
            name1, value1, name2, value2, name3, value3,
        );

        assert_eq!(json, expected);
    }

    #[test]
    fn test_or_with_multiple_gt_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);

        let query = Query::Or(vec![
            Query::Gt(name1.clone(), value1.clone()),
            Query::Gt(name2.clone(), value2.clone()),
            Query::Gt(name3.clone(), value3.clone()),
        ]);

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(
            r#"{{"$or":[{{"{}":{{"$gt":"{}"}}}},{{"{}":{{"$gt":"{}"}}}},{{"{}":{{"$gt":"{}"}}}}]}}"#,
            name1, value1, name2, value2, name3, value3,
        );

        assert_eq!(json, expected);
    }

    #[test]
    fn test_or_with_multiple_gte_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);

        let query = Query::Or(vec![
            Query::Gte(name1.clone(), value1.clone()),
            Query::Gte(name2.clone(), value2.clone()),
            Query::Gte(name3.clone(), value3.clone()),
        ]);

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(
            r#"{{"$or":[{{"{}":{{"$gte":"{}"}}}},{{"{}":{{"$gte":"{}"}}}},{{"{}":{{"$gte":"{}"}}}}]}}"#,
            name1, value1, name2, value2, name3, value3,
        );

        assert_eq!(json, expected);
    }

    #[test]
    fn test_or_with_multiple_lt_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);

        let query = Query::Or(vec![
            Query::Lt(name1.clone(), value1.clone()),
            Query::Lt(name2.clone(), value2.clone()),
            Query::Lt(name3.clone(), value3.clone()),
        ]);

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(
            r#"{{"$or":[{{"{}":{{"$lt":"{}"}}}},{{"{}":{{"$lt":"{}"}}}},{{"{}":{{"$lt":"{}"}}}}]}}"#,
            name1, value1, name2, value2, name3, value3,
        );

        assert_eq!(json, expected);
    }

    #[test]
    fn test_or_with_multiple_lte_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);

        let query = Query::Or(vec![
            Query::Lte(name1.clone(), value1.clone()),
            Query::Lte(name2.clone(), value2.clone()),
            Query::Lte(name3.clone(), value3.clone()),
        ]);

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(
            r#"{{"$or":[{{"{}":{{"$lte":"{}"}}}},{{"{}":{{"$lte":"{}"}}}},{{"{}":{{"$lte":"{}"}}}}]}}"#,
            name1, value1, name2, value2, name3, value3,
        );

        assert_eq!(json, expected);
    }

    #[test]
    fn test_or_with_multiple_like_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);

        let query = Query::Or(vec![
            Query::Like(name1.clone(), value1.clone()),
            Query::Like(name2.clone(), value2.clone()),
            Query::Like(name3.clone(), value3.clone()),
        ]);

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(
            r#"{{"$or":[{{"{}":{{"$like":"{}"}}}},{{"{}":{{"$like":"{}"}}}},{{"{}":{{"$like":"{}"}}}}]}}"#,
            name1, value1, name2, value2, name3, value3,
        );

        assert_eq!(json, expected);
    }

    #[test]
    fn test_or_with_multiple_in_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);

        let query = Query::Or(vec![
            Query::In(name1.clone(), vec![value1.clone()]),
            Query::In(name2.clone(), vec![value2.clone()]),
            Query::In(name3.clone(), vec![value3.clone()]),
        ]);

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(
            r#"{{"$or":[{{"{}":{{"$in":["{}"]}}}},{{"{}":{{"$in":["{}"]}}}},{{"{}":{{"$in":["{}"]}}}}]}}"#,
            name1, value1, name2, value2, name3, value3,
        );

        assert_eq!(json, expected);
    }

    #[test]
    fn test_or_with_multiple_not_eq_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);

        let query = Query::Or(vec![
            Query::Not(Box::new(Query::Eq(name1.clone(), value1.clone()))),
            Query::Not(Box::new(Query::Eq(name2.clone(), value2.clone()))),
            Query::Not(Box::new(Query::Eq(name3.clone(), value3.clone()))),
        ]);

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(
            r#"{{"$or":[{{"$not":{{"{}":"{}"}}}},{{"$not":{{"{}":"{}"}}}},{{"$not":{{"{}":"{}"}}}}]}}"#,
            name1, value1, name2, value2, name3, value3,
        );

        assert_eq!(json, expected);
    }

    #[test]
    fn test_or_with_multiple_mixed_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);
        let name4 = _random_string(10);
        let value4 = _random_string(10);
        let name5 = _random_string(10);
        let value5 = _random_string(10);
        let name6 = _random_string(10);
        let value6 = _random_string(10);
        let name7 = _random_string(10);
        let value7 = _random_string(10);
        let name8 = _random_string(10);
        let value8a = _random_string(10);
        let value8b = _random_string(10);
        let name9 = _random_string(10);
        let value9 = _random_string(10);

        let query = Query::Or(vec![
            Query::Eq(name1.clone(), value1.clone()),
            Query::Neq(name2.clone(), value2.clone()),
            Query::Gt(name3.clone(), value3.clone()),
            Query::Gte(name4.clone(), value4.clone()),
            Query::Lt(name5.clone(), value5.clone()),
            Query::Lte(name6.clone(), value6.clone()),
            Query::Like(name7.clone(), value7.clone()),
            Query::In(name8.clone(), vec![value8a.clone(), value8b.clone()]),
            Query::Not(Box::new(Query::Eq(name9.clone(), value9.clone()))),
        ]);

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(
            r#"{{"$or":[{{"{}":"{}"}},{{"{}":{{"$neq":"{}"}}}},{{"{}":{{"$gt":"{}"}}}},{{"{}":{{"$gte":"{}"}}}},{{"{}":{{"$lt":"{}"}}}},{{"{}":{{"$lte":"{}"}}}},{{"{}":{{"$like":"{}"}}}},{{"{}":{{"$in":["{}","{}"]}}}},{{"$not":{{"{}":"{}"}}}}]}}"#,
            name1,
            value1,
            name2,
            value2,
            name3,
            value3,
            name4,
            value4,
            name5,
            value5,
            name6,
            value6,
            name7,
            value7,
            name8,
            value8a,
            value8b,
            name9,
            value9,
        );

        assert_eq!(json, expected);
    }

    #[test]
    fn test_not_with_one_eq_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let query = Query::Not(Box::new(Query::Eq(name1.clone(), value1.clone())));

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"$not":{{"{}":"{}"}}}}"#, name1, value1);

        assert_eq!(json, expected);
    }

    #[test]
    fn test_not_with_one_neq_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let query = Query::Not(Box::new(Query::Neq(name1.clone(), value1.clone())));

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"$not":{{"{}":{{"$neq":"{}"}}}}}}"#, name1, value1);

        assert_eq!(json, expected);
    }

    #[test]
    fn test_not_with_one_gt_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let query = Query::Not(Box::new(Query::Gt(name1.clone(), value1.clone())));

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"$not":{{"{}":{{"$gt":"{}"}}}}}}"#, name1, value1);

        assert_eq!(json, expected);
    }

    #[test]
    fn test_not_with_one_gte_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let query = Query::Not(Box::new(Query::Gte(name1.clone(), value1.clone())));

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"$not":{{"{}":{{"$gte":"{}"}}}}}}"#, name1, value1);

        assert_eq!(json, expected);
    }

    #[test]
    fn test_not_with_one_lt_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let query = Query::Not(Box::new(Query::Lt(name1.clone(), value1.clone())));

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"$not":{{"{}":{{"$lt":"{}"}}}}}}"#, name1, value1);

        assert_eq!(json, expected);
    }

    #[test]
    fn test_not_with_one_lte_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let query = Query::Not(Box::new(Query::Lte(name1.clone(), value1.clone())));

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"$not":{{"{}":{{"$lte":"{}"}}}}}}"#, name1, value1);

        assert_eq!(json, expected);
    }

    #[test]
    fn test_not_with_one_like_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let query = Query::Not(Box::new(Query::Like(name1.clone(), value1.clone())));

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"$not":{{"{}":{{"$like":"{}"}}}}}}"#, name1, value1);

        assert_eq!(json, expected);
    }

    #[test]
    fn test_not_with_one_in_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let query = Query::Not(Box::new(Query::In(name1.clone(), vec![value1.clone()])));

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"$not":{{"{}":{{"$in":["{}"]}}}}}}"#, name1, value1);

        assert_eq!(json, expected);
    }

    #[test]
    fn test_and_or_not_complex_case_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);
        let name4 = _random_string(10);
        let value4 = _random_string(10);
        let name5 = _random_string(10);
        let value5 = _random_string(10);
        let name6 = _random_string(10);
        let value6 = _random_string(10);
        let name7 = _random_string(10);
        let value7 = _random_string(10);
        let name8 = _random_string(10);
        let value8 = _random_string(10);

        let query = Query::Not(Box::new(Query::And(vec![
            Query::Eq(name1.clone(), value1.clone()),
            Query::Or(vec![
                Query::Gt(name2.clone(), value2.clone()),
                Query::Not(Box::new(Query::Lte(name3.clone(), value3.clone()))),
                Query::And(vec![
                    Query::Lt(name4.clone(), value4.clone()),
                    Query::Not(Box::new(Query::Gte(name5.clone(), value5.clone()))),
                ]),
            ]),
            Query::Not(Box::new(Query::Like(name6.clone(), value6.clone()))),
            Query::And(vec![
                Query::Eq(name7.clone(), value7.clone()),
                Query::Not(Box::new(Query::Neq(name8.clone(), value8.clone()))),
            ]),
        ])));

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(
            r#"{{"$not":{{"$and":[{{"{}":"{}"}},{{"$or":[{{"{}":{{"$gt":"{}"}}}},{{"$not":{{"{}":{{"$lte":"{}"}}}}}},{{"$and":[{{"{}":{{"$lt":"{}"}}}},{{"$not":{{"{}":{{"$gte":"{}"}}}}}}]}}]}},{{"$not":{{"{}":{{"$like":"{}"}}}}}},{{"$and":[{{"{}":"{}"}},{{"$not":{{"{}":{{"$neq":"{}"}}}}}}]}}]}}}}"#,
            name1,
            value1,
            name2,
            value2,
            name3,
            value3,
            name4,
            value4,
            name5,
            value5,
            name6,
            value6,
            name7,
            value7,
            name8,
            value8,
        );

        assert_eq!(json, expected);
    }

    #[test]
    fn test_old_format() {
        let name1 = _random_string(10);
        let name2 = _random_string(10);
        let value1 = _random_string(10);
        let value2 = _random_string(10);

        let json = format!(
            r#"[{{"{}":"{}"}}, {{"{}":"{}"}}]"#,
            name1, value1, name2, value2
        );

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::Or(vec![Query::Eq(name1, value1), Query::Eq(name2, value2)]);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_old_format_empty() {
        let json = r#"[]"#;

        let query: Query = ::serde_json::from_str(json).unwrap();

        let expected = Query::And(vec![]);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_old_format_with_nulls() {
        let name1 = _random_string(10);
        let name2 = _random_string(10);
        let value1 = _random_string(10);

        let json = json!(vec![
            json!({ &name1: value1 }),
            json!({ name2: ::serde_json::Value::Null })
        ])
        .to_string();

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::Or(vec![Query::Eq(name1, value1)]);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_optimise_and() {
        let json = r#"{}"#;

        let query: Query = ::serde_json::from_str(json).unwrap();

        assert_eq!(query.optimise(), None);
    }

    #[test]
    fn test_optimise_or() {
        let json = r#"[]"#;

        let query: Query = ::serde_json::from_str(json).unwrap();

        assert_eq!(query.optimise(), None);
    }

    #[test]
    fn test_optimise_single_nested_and() {
        let json = json!({
            "$and": [
                {
                    "$and": []
                }
            ]
        })
        .to_string();

        let query: Query = ::serde_json::from_str(&json).unwrap();

        assert_eq!(query.optimise(), None);
    }

    #[test]
    fn test_optimise_several_nested_and() {
        let json = json!({
            "$and": [
                {
                    "$and": []
                },
                {
                    "$and": []
                }
            ]
        })
        .to_string();

        let query: Query = ::serde_json::from_str(&json).unwrap();

        assert_eq!(query.optimise(), None);
    }

    #[test]
    fn test_optimise_single_nested_or() {
        let json = json!({
            "$and": [
                {
                    "$or": []
                }
            ]
        })
        .to_string();

        let query: Query = ::serde_json::from_str(&json).unwrap();

        assert_eq!(query.optimise(), None);
    }

    #[test]
    fn test_optimise_several_nested_or() {
        let json = json!({
            "$and": [
                {
                    "$or": []
                },
                {
                    "$or": []
                }
            ]
        })
        .to_string();

        let query: Query = ::serde_json::from_str(&json).unwrap();

        assert_eq!(query.optimise(), None);
    }
}
