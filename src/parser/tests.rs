use super::*;

#[test]
fn test_valid_string_examples() {
    for s in vec![
        ("\"\\uD834\\uDd1e\"", "ab"),
        ("\"asd\"", "asd"),
        ("\"as asd  asd d\\\"\"", "as asd  asd d\""),
        ("\"asd\\r\\n\\t\"", "asd\r\n\t"),
        ("\"\\u0041\"", "A"),
        ("\"unicode sequence \\uc328\"", "unicode sequence 쌨"),
    ] {
        assert_eq!(parse_str(&mut s.0.char_indices().peekable()).unwrap(), s.1);
    }
}

#[test]
fn test_invalid_string_examples() {
    for s in vec![
        ("\"this \n fails \""),
        ("no quotes"),
        ("\"not_closed"),
        ("not opened"),
        ("\"invalid escape \\x \""),
    ] {
        parse_str(&mut s.char_indices().peekable())
            .expect_err(&format!("Invalid value {} parsed", s));
    }
}

#[test]
fn valid_parse_bull() {
    for s in vec!["true", "true, ", "true  asdpjmklmo"] {
        assert!(parse_true(&mut s.char_indices().peekable()).unwrap())
    }
    for s in vec!["false", "false, ", "false  asdpjmklmo"] {
        assert!(!parse_false(&mut s.char_indices().peekable()).unwrap())
    }
}

#[test]
fn invalid_parse_bull() {
    for s in vec!["True", "False", "TRUE", "0", "1", "asdm"] {
        parse_true(&mut s.char_indices().peekable())
            .expect_err(&format!("Should not be parsed as bool! {}", s));
        parse_false(&mut s.char_indices().peekable())
            .expect_err(&format!("Should not be parsed as bool! {}", s));
    }
}

#[test]
fn test_valid_parse_num() {
    for s in vec![
        ("0e1", 0.0),
        ("1,2", 1.0),
        ("1}", 1.0),
        ("1,", 1.0),
        ("123", 123.0),
        ("113.1", 113.1),
        ("0.542", 0.542),
        ("0.0000000001", 0.0000000001),
        (
            "12312518359823.23482394823930113570185103857",
            12312518359823.23482394823930113570185103857,
        ),
        ("0.00E+123", 0.0),
        ("-123123123123123.1291", -123123123123123.1291),
        ("0.1212E9", 0.1212E9),
        ("0.1212E+100", 0.1212E100),
        ("1231231239.0121e-121", 1231231239.0121e-121),
        ("1231231239.0121e-5000 asd", 1231231239.0121e-5000),
    ] {
        println!("Checking {}", s.0);
        assert_eq!(parse_num(&mut s.0.char_indices().peekable()).unwrap(), s.1)
    }
}

#[test]
fn test_invalid_parse_num() {
    for s in vec![
        "a123",
        "0.e1",
        "00.123",
        "+123",
        "a1u2djasjda",
        "123.0Ee123123123",
    ] {
        println!("Checking {}", s);
        parse_num(&mut s.char_indices().peekable())
            .expect_err(&format!("Expected to fail while parsing {}", s));
    }
}

#[test]
fn test_valid_parse_null() {
    for s in vec!["null", "null, ", "null ", "null!"] {
        parse_null(&mut s.char_indices().peekable()).unwrap();
    }
}

#[test]
fn invalid_parse_null() {
    for s in vec!["NULL", "!null", "asd", "><>OP"] {
        parse_null(&mut s.char_indices().peekable())
            .expect_err(&format!("Should not be parsed as null! {}", s));
    }
}

#[test]
fn test_invalid_parse_object() {
    for s in vec![
        "{,}",
        "{1 : 1}",
        "{\"asd\": 1,}",
        "{\"asd\"; 1}",
        "{\"asd\": 1",
        "\"asd\": 1}",
        "{\"asd\": 1; \"bsd\": 2}",
        "{\"asd\": 1; \"bsd\": \"asdasdad}",
    ] {
        parse_object(&mut s.char_indices().peekable())
            .expect_err(&format!("Should not be parsed as valid object <{}>", s));
    }
}

#[test]
fn test_valid_parse_object() {
    for s in vec![
        "{}",
        "{\"asd\": 1}",
        "{\"asd\": {\"b\": 1}}",
        "{\"asd\": 17.8e162}",
        "{\"asd\": 1, \"bsd\": 2}",
        "{\"asd\": 1, \"bsd\": \"asdasdasd\"}",
    ] {
        println!("Checking {}", s);
        parse_object(&mut s.char_indices().peekable()).unwrap();
    }
}

#[test]
fn test_valid_parse_array() {
    for s in vec![
        (
            "[1,2,3]",
            vec![
                Box::new(JSONValue::JSONNumber(1.0)),
                Box::new(JSONValue::JSONNumber(2.0)),
                Box::new(JSONValue::JSONNumber(3.0)),
            ],
        ),
        (
            "[1, 2, 3.0]",
            vec![
                Box::new(JSONValue::JSONNumber(1.0)),
                Box::new(JSONValue::JSONNumber(2.0)),
                Box::new(JSONValue::JSONNumber(3.0)),
            ],
        ),
        (
            "[1, 2, [1,     2,              3]]",
            vec![
                Box::new(JSONValue::JSONNumber(1.0)),
                Box::new(JSONValue::JSONNumber(2.0)),
                Box::new(JSONValue::JSONArray(vec![
                    Box::new(JSONValue::JSONNumber(1.0)),
                    Box::new(JSONValue::JSONNumber(2.0)),
                    Box::new(JSONValue::JSONNumber(3.0)),
                ])),
            ],
        ),
        (
            "[     1,2,3      ]",
            vec![
                Box::new(JSONValue::JSONNumber(1.0)),
                Box::new(JSONValue::JSONNumber(2.0)),
                Box::new(JSONValue::JSONNumber(3.0)),
            ],
        ),
        (
            "[null, 1, \"1\", {}]",
            vec![
                Box::new(JSONValue::JSONNull()),
                Box::new(JSONValue::JSONNumber(1.0)),
                Box::new(JSONValue::JSONString("1".to_owned())),
                Box::new(JSONValue::JSONObject(HashMap::new())),
            ],
        ),
    ] {
        println!("Checking {}", s.0);
        assert_eq!(
            parse_array(&mut s.0.char_indices().peekable()).unwrap(),
            s.1
        );
    }
}

#[test]
fn test_invalid_parse_array() {
    let cases = vec![
        "[\x0c]",
        "[invalidelement]",
        "[asd",
        "asd]",
        "asd]",
        "[1, 2, 3 4]",
    ];
    for s in cases {
        parse_array(&mut s.char_indices().peekable())
            .expect_err(&format!("Should not be parsed as valid array <{}>", s));
    }
}
