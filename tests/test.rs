mod prettyprinter;

use std::fs;

use actson::feeder::PushJsonFeeder;
use actson::{JsonEvent, JsonParser};
use prettyprinter::PrettyPrinter;
use serde_json::Value;

/// Parse a JSON string and return a new JSON string generated by
/// [`PrettyPrinter`]. Assert that the input JSON string is valid.
fn parse(json: &str) -> String {
    let buf = json.as_bytes();

    let mut prettyprinter = PrettyPrinter::new();
    let mut feeder = PushJsonFeeder::new();
    let mut parser = JsonParser::new();
    let mut i: usize = 0;
    loop {
        // feed as many bytes as possible to the parser
        let mut e = parser.next_event(&mut feeder);
        while e == JsonEvent::NeedMoreInput {
            i += feeder.push_bytes(&buf[i..]);
            if i == json.len() {
                feeder.done();
            }
            e = parser.next_event(&mut feeder);
        }

        assert_ne!(e, JsonEvent::Error);

        prettyprinter.on_event(e, &parser).unwrap();

        if e == JsonEvent::Eof {
            break;
        }
    }

    prettyprinter.get_result().to_string()
}

/// Parse a JSON string and expect parsing to fail
fn parse_fail(json: &str) {
    parse_fail_with_parser(json, &mut JsonParser::new());
}

fn parse_fail_with_parser(json: &str, parser: &mut JsonParser) {
    let buf = json.as_bytes();

    let mut feeder = PushJsonFeeder::new();
    let mut i: usize = 0;
    let mut ok: bool;
    loop {
        // feed as many bytes as possible to the parser
        let mut e = parser.next_event(&mut feeder);
        while e == JsonEvent::NeedMoreInput {
            i += feeder.push_bytes(&buf[i..]);
            if i == json.len() {
                feeder.done();
            }
            e = parser.next_event(&mut feeder);
        }

        ok = e != JsonEvent::Error;

        if !ok || e == JsonEvent::Eof {
            break;
        }
    }

    assert!(!ok);
}

fn assert_json_eq(expected: &str, actual: &str) {
    let em: Value = serde_json::from_str(expected).unwrap();
    let am: Value = serde_json::from_str(actual).unwrap();
    assert_eq!(em, am);
}

/// Test if valid files can be parsed correctly
#[test]
fn test_pass() {
    for i in 1..=3 {
        let json = fs::read_to_string(format!("tests/fixtures/pass{}.txt", i)).unwrap();
        assert_json_eq(&json, &parse(&json));
    }
}

#[test]
fn test_fail() {
    let mut parser = JsonParser::new_with_max_depth(16);
    for i in 2..=34 {
        let json = fs::read_to_string(format!("tests/fixtures/fail{}.txt", i)).unwrap();
        parse_fail_with_parser(&json, &mut parser);
    }
}

/// Test that an empty object is parsed correctly
#[test]
fn empty_object() {
    let json = r#"{}"#;
    assert_json_eq(json, &parse(json));
}

/// Test that a simple object is parsed correctly
#[test]
fn simple_object() {
    let json = r#"{"name": "Elvis"}"#;
    assert_json_eq(json, &parse(json));
}

/// Test that an empty array is parsed correctly
#[test]
fn empty_array() {
    let json = r#"[]"#;
    assert_json_eq(json, &parse(json));
}

/// Test that a simple array is parsed correctly
#[test]
fn simple_array() {
    let json = r#"["Elvis", "Max"]"#;
    assert_json_eq(json, &parse(json));
}

/// Test that an array with mixed values is parsed correctly
#[test]
fn mixed_array() {
    let json = r#"["Elvis", 132, "Max", 80.67]"#;
    assert_json_eq(json, &parse(json));
}

/// Test that a JSON text containing a UTF-8 character is parsed correctly
#[test]
fn utf8() {
    let json = "{\"name\": \"Bj\u{0153}rn\"}";
    assert_json_eq(json, &parse(json));
}

#[test]
fn too_many_next_event() {
    let json = "{}";
    let mut parser = JsonParser::new();
    assert_json_eq(json, &parse(json));

    let mut feeder = PushJsonFeeder::new();
    feeder.done();
    assert_eq!(parser.next_event(&mut feeder), JsonEvent::Error);
}

/// Make sure a number right before the end of the object can be parsed
#[test]
fn number_and_end_of_object() {
    let json = r#"{"n":2}"#;
    assert_json_eq(json, &parse(json));
}

/// Make sure a fraction can be parsed
#[test]
fn fraction() {
    let json = r#"{"n":2.1}"#;
    assert_json_eq(json, &parse(json));
}

/// Test that the parser does not accept illegal numbers ending with a dot
#[test]
fn illegal_number() {
    let json = r#"{"n":-2.}"#;
    parse_fail(json);
}

/// Make sure '0e1' can be parsed
#[test]
fn zero_with_exp() {
    let json = r#"{"n":0e1}"#;
    assert_json_eq(json, &parse(json));
}

/// Test if a top-level empty string can be parsed
#[test]
fn top_level_empty_string() {
    let json = r#""""#;
    assert_json_eq(json, &parse(json));
}

/// Test if a top-level 'false' can be parsed
#[test]
fn top_level_false() {
    let json = r#"false"#;
    assert_json_eq(json, &parse(json));
}

/// Test if a top-level integer can be parsed
#[test]
fn top_level_int() {
    let json = r#"42"#;
    assert_json_eq(json, &parse(json));
}

/// Test if a top-level long can be parsed
#[test]
fn top_level_long() {
    let json = r#"42123123123"#;
    assert_json_eq(json, &parse(json));
}

/// Make sure pre-mature end of file is detected correctly
#[test]
fn number_and_eof() {
    let json = r#"{"i":42"#;
    parse_fail(json);
}

/// Test if a top-level zero can be parsed
#[test]
fn top_level_zero() {
    let json = r#"0"#;
    assert_json_eq(json, &parse(json));
}
