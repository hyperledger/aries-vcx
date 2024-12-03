use std::sync::Once;

use did_parser_nom::Did;

static TEST_LOGGING_INIT: Once = Once::new();

pub fn init_logger() {
    TEST_LOGGING_INIT.call_once(|| {
        let env = env_logger::Env::default().default_filter_or("info");
        env_logger::init_from_env(env);
    })
}

macro_rules! test_cases_positive {
    ($($name:ident: $input_did:expr, $expected_method:expr, $expected_namespace:expr, $expected_id:expr)*) => {
        $(
            #[test]
            fn $name() {
                init_logger();

                log::debug!("Testing parsing of {}", $input_did);
                let parsed_did = Did::parse($input_did.to_string()).unwrap();

                assert_eq!(parsed_did.did(), $input_did, "DID");
                assert_eq!(parsed_did.method(), $expected_method, "Method");
                assert_eq!(parsed_did.namespace(), $expected_namespace, "Namespace");
                assert_eq!(parsed_did.id(), $expected_id, "ID");
            }
        )*
    };
}

test_cases_positive! {
    test_did_unknown_method:
        "did:example:123456789abcdefghi",
        Some("example"),
        None,
        "123456789abcdefghi"
    test_did_web:
        "did:web:w3c-ccg.github.io",
        Some("web"),
        None,
        "w3c-ccg.github.io"
    test_did_sov_unqualified:
        "2ZHFFhzA2XtTD6hJqzL7ux",
        None,
        None,
        "2ZHFFhzA2XtTD6hJqzL7ux"
    test_did_sov:
        "did:sov:2wJPyULfLLnYTEFYzByfUR",
        Some("sov"),
        None,
        "2wJPyULfLLnYTEFYzByfUR"
    test_did_key:
        "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
        Some("key"),
        None,
        "z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK"
    test_did_indy:
        "did:indy:sovrin:7Tqg6BwSSWapxgUDm9KKgg",
        Some("indy"),
        None,
        "sovrin:7Tqg6BwSSWapxgUDm9KKgg"
    test_did_sov_namespaced:
        "did:sov:builder:VbPQNHsvoLZdaNU7fTBeFx",
        Some("sov"),
        Some("builder"),
        "VbPQNHsvoLZdaNU7fTBeFx"
    test_did_peer_2:
        "did:peer:2.Vz6Mkj3PUd1WjvaDhNZhhhXQdz5UnZXmS7ehtx8bsPpD47kKc.Ez6LSg8zQom395jKLrGiBNruB9MM6V8PWuf2FpEy4uRFiqQBR.SeyJ0IjoiZG0iLCJzIjp7InVyaSI6Imh0dHA6Ly9leGFtcGxlLmNvbS9kaWRjb21tIiwiYSI6WyJkaWRjb21tL3YyIl0sInIiOlsiZGlkOmV4YW1wbGU6MTIzNDU2Nzg5YWJjZGVmZ2hpI2tleS0xIl19fQ.SeyJ0IjoiZG0iLCJzIjp7InVyaSI6Imh0dHA6Ly9leGFtcGxlLmNvbS9hbm90aGVyIiwiYSI6WyJkaWRjb21tL3YyIl0sInIiOlsiZGlkOmV4YW1wbGU6MTIzNDU2Nzg5YWJjZGVmZ2hpI2tleS0yIl19fQ",
        Some("peer"),
        None,
        "2.Vz6Mkj3PUd1WjvaDhNZhhhXQdz5UnZXmS7ehtx8bsPpD47kKc.Ez6LSg8zQom395jKLrGiBNruB9MM6V8PWuf2FpEy4uRFiqQBR.SeyJ0IjoiZG0iLCJzIjp7InVyaSI6Imh0dHA6Ly9leGFtcGxlLmNvbS9kaWRjb21tIiwiYSI6WyJkaWRjb21tL3YyIl0sInIiOlsiZGlkOmV4YW1wbGU6MTIzNDU2Nzg5YWJjZGVmZ2hpI2tleS0xIl19fQ.SeyJ0IjoiZG0iLCJzIjp7InVyaSI6Imh0dHA6Ly9leGFtcGxlLmNvbS9hbm90aGVyIiwiYSI6WyJkaWRjb21tL3YyIl0sInIiOlsiZGlkOmV4YW1wbGU6MTIzNDU2Nzg5YWJjZGVmZ2hpI2tleS0yIl19fQ"
    test_did_peer_3:
        "did:peer:3zQmS19jtYDvGtKVrJhQnRFpBQAx3pJ9omx2HpNrcXFuRCz9",
        Some("peer"),
        None,
        "3zQmS19jtYDvGtKVrJhQnRFpBQAx3pJ9omx2HpNrcXFuRCz9"
    test_did_peer_4:
        "did:peer:4z84UjLJ6ugExV8TJ5gJUtZap5q67uD34LU26m1Ljo2u9PZ4xHa9XnknHLc3YMST5orPXh3LKi6qEYSHdNSgRMvassKP:z27uFkiqJVwvvn2ke5M19UCvByS79r5NppqwjiGAJzkj1EM4sf2JmiUySkANKy4YNu8M7yKjSmvPJTqbcyhPrJs9TASzDs2fWE1vFegmaRJxHRF5M9wGTPwGR1NbPkLGsvcnXum7aN2f8kX3BnhWWWp",
        Some("peer"),
        None,
        "4z84UjLJ6ugExV8TJ5gJUtZap5q67uD34LU26m1Ljo2u9PZ4xHa9XnknHLc3YMST5orPXh3LKi6qEYSHdNSgRMvassKP:z27uFkiqJVwvvn2ke5M19UCvByS79r5NppqwjiGAJzkj1EM4sf2JmiUySkANKy4YNu8M7yKjSmvPJTqbcyhPrJs9TASzDs2fWE1vFegmaRJxHRF5M9wGTPwGR1NbPkLGsvcnXum7aN2f8kX3BnhWWWp"
    test_did_cheqd:
        "did:cheqd:mainnet:de9786cd-ec53-458c-857c-9342cf264f80",
        Some("cheqd"),
        Some("mainnet"),
        "de9786cd-ec53-458c-857c-9342cf264f80"
    test_did_cheqd_indy_style:
        "did:cheqd:testnet:TAwT8WVt3dz2DBAifwuSkn",
        Some("cheqd"),
        Some("testnet"),
        "TAwT8WVt3dz2DBAifwuSkn"
}
