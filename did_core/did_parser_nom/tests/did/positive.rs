use did_parser_nom::Did;

macro_rules! test_cases_positive {
    ($($name:ident: $input:expr, $expected_did:expr, $expected_method:expr, $expected_namespace:expr, $expected_id:expr)*) => {
        $(
            #[test]
            fn $name() {
                println!("Testing {}", $input);
                let parsed_did = Did::parse($input.to_string()).unwrap();

                assert_eq!(parsed_did.did(), $expected_did, "DID");
                assert_eq!(parsed_did.method(), $expected_method, "Method");
                assert_eq!(parsed_did.namespace(), $expected_namespace, "Namespace");
                assert_eq!(parsed_did.id(), $expected_id, "ID");
            }
        )*
    };
}

test_cases_positive! {
    test_case1:
        "did:example:123456789abcdefghi",
        "did:example:123456789abcdefghi",
        Some("example"),
        None,
        "123456789abcdefghi"
    test_case2:
        "did:web:w3c-ccg.github.io",
        "did:web:w3c-ccg.github.io",
        Some("web"),
        None,
        "w3c-ccg.github.io"
    test_case3:
        "2ZHFFhzA2XtTD6hJqzL7ux",
        "2ZHFFhzA2XtTD6hJqzL7ux",
        None,
        None,
        "2ZHFFhzA2XtTD6hJqzL7ux"
    test_case4:
        "did:sov:2wJPyULfLLnYTEFYzByfUR",
        "did:sov:2wJPyULfLLnYTEFYzByfUR",
        Some("sov"),
        None,
        "2wJPyULfLLnYTEFYzByfUR"
    test_case5:
        "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
        "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
        Some("key"),
        None,
        "z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK"
    test_case6:
        "did:indy:sovrin:7Tqg6BwSSWapxgUDm9KKgg",
        "did:indy:sovrin:7Tqg6BwSSWapxgUDm9KKgg",
        Some("indy"),
        Some("sovrin"),
        "7Tqg6BwSSWapxgUDm9KKgg"
    test_case7:
        "did:indy:sovrin:alpha:7Tqg6BwSSWapxgUDm9KKgg",
        "did:indy:sovrin:alpha:7Tqg6BwSSWapxgUDm9KKgg",
        Some("indy"),
        Some("sovrin:alpha"),
        "7Tqg6BwSSWapxgUDm9KKgg"
    test_case8:
        "did:indy:sovrin:alpha:%0Aqg6BwS.Wapxg-Dm9K_gg",
        "did:indy:sovrin:alpha:%0Aqg6BwS.Wapxg-Dm9K_gg",
        Some("indy"),
        Some("sovrin:alpha"),
        "%0Aqg6BwS.Wapxg-Dm9K_gg"
    test_case9:
        "did:sov:builder:VbPQNHsvoLZdaNU7fTBeFx",
        "did:sov:builder:VbPQNHsvoLZdaNU7fTBeFx",
        Some("sov"),
        Some("builder"),
        "VbPQNHsvoLZdaNU7fTBeFx"
    test_case10:
        "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
        "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
        Some("key"),
        None,
        "z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK"
    test_case11:
        "did:peer:2.Vz6Mkj3PUd1WjvaDhNZhhhXQdz5UnZXmS7ehtx8bsPpD47kKc.Ez6LSg8zQom395jKLrGiBNruB9MM6V8PWuf2FpEy4uRFiqQBR.SeyJ0IjoiZG0iLCJzIjp7InVyaSI6Imh0dHA6Ly9leGFtcGxlLmNvbS9kaWRjb21tIiwiYSI6WyJkaWRjb21tL3YyIl0sInIiOlsiZGlkOmV4YW1wbGU6MTIzNDU2Nzg5YWJjZGVmZ2hpI2tleS0xIl19fQ.SeyJ0IjoiZG0iLCJzIjp7InVyaSI6Imh0dHA6Ly9leGFtcGxlLmNvbS9hbm90aGVyIiwiYSI6WyJkaWRjb21tL3YyIl0sInIiOlsiZGlkOmV4YW1wbGU6MTIzNDU2Nzg5YWJjZGVmZ2hpI2tleS0yIl19fQ",
        "did:peer:2.Vz6Mkj3PUd1WjvaDhNZhhhXQdz5UnZXmS7ehtx8bsPpD47kKc.Ez6LSg8zQom395jKLrGiBNruB9MM6V8PWuf2FpEy4uRFiqQBR.SeyJ0IjoiZG0iLCJzIjp7InVyaSI6Imh0dHA6Ly9leGFtcGxlLmNvbS9kaWRjb21tIiwiYSI6WyJkaWRjb21tL3YyIl0sInIiOlsiZGlkOmV4YW1wbGU6MTIzNDU2Nzg5YWJjZGVmZ2hpI2tleS0xIl19fQ.SeyJ0IjoiZG0iLCJzIjp7InVyaSI6Imh0dHA6Ly9leGFtcGxlLmNvbS9hbm90aGVyIiwiYSI6WyJkaWRjb21tL3YyIl0sInIiOlsiZGlkOmV4YW1wbGU6MTIzNDU2Nzg5YWJjZGVmZ2hpI2tleS0yIl19fQ",
        Some("peer"),
        None,
        "2.Vz6Mkj3PUd1WjvaDhNZhhhXQdz5UnZXmS7ehtx8bsPpD47kKc.Ez6LSg8zQom395jKLrGiBNruB9MM6V8PWuf2FpEy4uRFiqQBR.SeyJ0IjoiZG0iLCJzIjp7InVyaSI6Imh0dHA6Ly9leGFtcGxlLmNvbS9kaWRjb21tIiwiYSI6WyJkaWRjb21tL3YyIl0sInIiOlsiZGlkOmV4YW1wbGU6MTIzNDU2Nzg5YWJjZGVmZ2hpI2tleS0xIl19fQ.SeyJ0IjoiZG0iLCJzIjp7InVyaSI6Imh0dHA6Ly9leGFtcGxlLmNvbS9hbm90aGVyIiwiYSI6WyJkaWRjb21tL3YyIl0sInIiOlsiZGlkOmV4YW1wbGU6MTIzNDU2Nzg5YWJjZGVmZ2hpI2tleS0yIl19fQ"
    test_case12:
        "did:peer:3zQmS19jtYDvGtKVrJhQnRFpBQAx3pJ9omx2HpNrcXFuRCz9",
        "did:peer:3zQmS19jtYDvGtKVrJhQnRFpBQAx3pJ9omx2HpNrcXFuRCz9",
        Some("peer"),
        None,
        "3zQmS19jtYDvGtKVrJhQnRFpBQAx3pJ9omx2HpNrcXFuRCz9"
}
