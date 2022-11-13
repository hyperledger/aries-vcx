pub static REQUESTED_ATTRS: &str = r#"[
    {
        "name": "age",
        "restrictions": [
            {
                "schema_id": "6XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11",
                "schema_name": "Faber Student Info",
                "schema_version": "1.0",
                "schema_issuer_did": "6XFh8yBzrpJQmNyZzgoTqB",
                "issuer_did": "8XFh8yBzrpJQmNyZzgoTqB",
                "cred_def_id": "8XFh8yBzrpJQmNyZzgoTqB:3:CL:1766"
            },
            {
                "schema_id": "5XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11",
                "schema_name": "BYU Student Info",
                "schema_version": "1.0",
                "schema_issuer_did": "5XFh8yBzrpJQmNyZzgoTqB",
                "issuer_did": "66Fh8yBzrpJQmNyZzgoTqB",
                "cred_def_id": "66Fh8yBzrpJQmNyZzgoTqB:3:CL:1766"
            }
        ]
    },
    {
        "name": "name",
        "restrictions": [
            {
                "schema_id": "6XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11",
                "schema_name": "Faber Student Info",
                "schema_version": "1.0",
                "schema_issuer_did": "6XFh8yBzrpJQmNyZzgoTqB",
                "issuer_did": "8XFh8yBzrpJQmNyZzgoTqB",
                "cred_def_id": "8XFh8yBzrpJQmNyZzgoTqB:3:CL:1766"
            },
            {
                "schema_id": "5XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11",
                "schema_name": "BYU Student Info",
                "schema_version": "1.0",
                "schema_issuer_did": "5XFh8yBzrpJQmNyZzgoTqB",
                "issuer_did": "66Fh8yBzrpJQmNyZzgoTqB",
                "cred_def_id": "66Fh8yBzrpJQmNyZzgoTqB:3:CL:1766"
            }
        ]
    }
]"#;
pub static REQUESTED_PREDICATES: &str = r#"[
    {
        "name": "age",
        "p_type": "GE",
        "p_value": 22,
        "restrictions": [
            {
                "schema_id": "6XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11",
                "schema_name": "Faber Student Info",
                "schema_version": "1.0",
                "schema_issuer_did": "6XFh8yBzrpJQmNyZzgoTqB",
                "issuer_did": "8XFh8yBzrpJQmNyZzgoTqB",
                "cred_def_id": "8XFh8yBzrpJQmNyZzgoTqB:3:CL:1766"
            },
            {
                "schema_id": "5XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11",
                "schema_name": "BYU Student Info",
                "schema_version": "1.0",
                "schema_issuer_did": "5XFh8yBzrpJQmNyZzgoTqB",
                "issuer_did": "66Fh8yBzrpJQmNyZzgoTqB",
                "cred_def_id": "66Fh8yBzrpJQmNyZzgoTqB:3:CL:1766"
            }
        ]
    }
]"#;
pub static ATTR_INFO_1: &str = r#"
{
    "name": "age",
    "restrictions": [
        {
            "schema_id": "6XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11",
            "schema_name": "Faber Student Info",
            "schema_version": "1.0",
            "schema_issuer_did": "6XFh8yBzrpJQmNyZzgoTqB",
            "issuer_did": "8XFh8yBzrpJQmNyZzgoTqB",
            "cred_def_id": "8XFh8yBzrpJQmNyZzgoTqB:3:CL:1766"
        },
        {
            "schema_id": "5XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11",
            "schema_name": "BYU Student Info",
            "schema_version": "1.0",
            "schema_issuer_did": "5XFh8yBzrpJQmNyZzgoTqB",
            "issuer_did": "66Fh8yBzrpJQmNyZzgoTqB",
            "cred_def_id": "66Fh8yBzrpJQmNyZzgoTqB:3:CL:1766"
        }
    ]
}"#;

pub static ATTR_INFO_2: &str = r#"
{
    "name": "name",
    "restrictions": [
        {
            "schema_id": "6XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11",
            "schema_name": "Faber Student Info",
            "schema_version": "1.0",
            "schema_issuer_did": "6XFh8yBzrpJQmNyZzgoTqB",
            "issuer_did": "8XFh8yBzrpJQmNyZzgoTqB",
            "cred_def_id": "8XFh8yBzrpJQmNyZzgoTqB:3:CL:1766"
        },
        {
            "schema_id": "5XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11",
            "schema_name": "BYU Student Info",
            "schema_version": "1.0",
            "schema_issuer_did": "5XFh8yBzrpJQmNyZzgoTqB",
            "issuer_did": "66Fh8yBzrpJQmNyZzgoTqB",
            "cred_def_id": "66Fh8yBzrpJQmNyZzgoTqB:3:CL:1766"
        }
    ]
}"#;
