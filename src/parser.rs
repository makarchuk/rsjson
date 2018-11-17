use super::*;
use std::char;
use std::iter::Peekable;
use std::str::CharIndices;

#[test]
fn test_parser() {
    // parse_json("{}").unwrap();
    // parse_json("{\"asd\": \"qweqweqwe\"}").unwrap();
}

const ESCAPE: char = '\\';
const OBJECT_START: char = '{';
const OBJECT_END: char = '}';
const ARRAY_START: char = '[';
const ARRAY_END: char = ']';
const COMMA: char = ',';
const COLON: char = ':';
const MINUS: char = '-';
const PLUS: char = '+';
const QUOTE: char = '\"';
const UNICODE_ESCAPE: char = 'u';
const TRUE_START: char = 't';
const FALSE_START: char = 'f';
const NULL_START: char = 'n';
const NULL: &str = "null";
const BOOL_TRUE: &str = "true";
const BOOL_FALSE: &str = "false";
const ESCAPABLE: &str = "\"\\/fnrtb";

const ERROR_ENDED_UNEXPECTEDLY: &str = "String ended unexpectedly";

pub fn parse_json(input: &str) -> Result<JSONValue, JSONParseError> {
    let mut chars = input.char_indices().peekable();
    consume_spaces(&mut chars);
    let val = parse_value(&mut chars)?;
    consume_spaces(&mut chars);
    unimplemented!();
}

pub fn parse_value(chars: &mut Peekable<CharIndices>) -> Result<JSONValue, JSONParseError> {
    let fun = match next_char(chars) {
        None => return Err(make_err("Empty string provided".to_owned())),
        Some(ch) => match ch {
            OBJECT_START => parse_object,
            QUOTE => parse_str,
            TRUE_START => parse_true,
            FALSE_START => parse_false,
            NULL_START => parse_null,
            MINUS => parse_num,
            '0'...'9' => parse_num,
            _ => unimplemented!(),
        },
    };
    let res = fun(chars)?;
    return Ok(res);
}

fn parse_const(
    chars: &mut Peekable<CharIndices>,
    str_value: &str,
    value: JSONValue,
) -> Result<JSONValue, JSONParseError> {
    for correct_char in str_value.chars() {
        let (i, ch) = chars.next().ok_or(unexpected_eof())?;
        if correct_char != ch {
            return Err(unexpected_charachter(i, ch));
        }
    }
    return Ok(value);
}

fn parse_true(chars: &mut Peekable<CharIndices>) -> Result<JSONValue, JSONParseError> {
    return parse_const(chars, BOOL_TRUE, JSONValue::JSONBool(true));
}

fn parse_false(chars: &mut Peekable<CharIndices>) -> Result<JSONValue, JSONParseError> {
    return parse_const(chars, BOOL_FALSE, JSONValue::JSONBool(false));
}

fn parse_null(chars: &mut Peekable<CharIndices>) -> Result<JSONValue, JSONParseError> {
    return parse_const(chars, NULL, JSONValue::JSONNull());
}

fn parse_object(chars: &mut Peekable<CharIndices>) -> Result<JSONValue, JSONParseError> {
    unimplemented!()
}

fn parse_str(chars: &mut Peekable<CharIndices>) -> Result<JSONValue, JSONParseError> {
    let mut result = String::new();
    let (i, ch) = chars.next().ok_or(unexpected_eof())?;
    if ch != '"' {
        return Err(unexpected_charachter(i, ch));
    }
    loop {
        let (_, ch) = chars.next().ok_or(unexpected_eof())?;
        match ch {
            QUOTE => return Ok(JSONValue::JSONString(result)),
            ESCAPE => result.push_str(&read_escape_char(chars)?),
            _ => result.push(ch),
        }
    }
}

fn read_escape_char(chars: &mut Peekable<CharIndices>) -> Result<String, JSONParseError> {
    let mut result = String::new();
    let (i, ch) = chars.next().ok_or(unexpected_eof())?;
    if ESCAPABLE.chars().any(|escapable| escapable == ch) {
        result.push(convert_escaped(ch));
    } else {
        if ch == UNICODE_ESCAPE {
            let mut ord: u32 = 0;
            let mut seq = "\\u".to_owned();
            for j in 0..4 {
                let (i, ch) = chars.next().ok_or(unexpected_eof())?;
                seq.push(ch);
                ord = ord * 16 + ch
                    .to_digit(16)
                    .ok_or(invalid_escape_sequence(i - j - 2, &seq))?;
            }
            result.push(char::from_u32(ord).ok_or(invalid_escape_sequence(i - 2, &seq))?)
        } else {
            return Err(invalid_escape_sequence(i - 2, &format!("\\{}", ch)));
        }
    }
    Ok(result)
}

fn convert_escaped(ch: char) -> char {
    match ch {
        't' => '\t',
        'r' => '\r',
        'n' => '\n',
        'f' => '\x0c',
        'b' => '\x08',
        _ => ch,
    }
}

fn parse_num(chars: &mut Peekable<CharIndices>) -> Result<JSONValue, JSONParseError> {
    let mut num = String::new();
    let ch = next_char(chars).ok_or(unexpected_eof())?;
    if ch == MINUS {
        num.push(ch);
        chars.next();
    }
    let ch = next_char(chars).ok_or(unexpected_eof())?;
    match ch {
        '0' => {
            num.push(ch);
            chars.next();
        }
        '1'...'9' => num.push_str(&read_digits(chars)?),
        _ => {
            let (i, ch) = chars.next().ok_or(unexpected_eof())?;
            return Err(unexpected_charachter(i, ch));
        }
    }
    num.push_str(&read_fraction(chars)?);
    match chars.next() {
        None => (),
        Some(el) => {
            let (i, ch) = el;
            match ch {
                'e' | 'E' => {
                    num.push(ch);
                    let ch = next_char(chars).ok_or(unexpected_eof())?;
                    match ch {
                        MINUS => {
                            num.push(ch);
                            chars.next();
                        }
                        PLUS => {
                            chars.next();
                        }
                        _ => (),
                    }
                    println!("Num so far: {}", num);
                    num.push_str(&read_digits(chars)?);
                }
                _ => return Err(unexpected_charachter(i, ch)),
            }
        }
    }
    match num.parse() {
        Ok(n) => return Ok(JSONValue::JSONNumber(n)),
        Err(_) => return Err(make_err(format!("Unable to parse number {}", num))),
    }
}

fn read_digits(chars: &mut Peekable<CharIndices>) -> Result<String, JSONParseError> {
    let mut result = String::new();
    let mut should_advance = true;
    loop {
        match next_char(chars) {
            None => {
                if result.len() > 0 {
                    return Ok(result);
                }
                return Err(unexpected_eof());
            }
            Some(ch) => {
                if ch.is_digit(10) {
                    result.push(ch);
                    chars.next();
                } else {
                    break;
                }
            }
        }
    }
    return Ok(result);
}

fn read_fraction(chars: &mut Peekable<CharIndices>) -> Result<String, JSONParseError> {
    match next_char(chars) {
        None => return Ok(String::new()),
        Some(ch) => {
            if ch == '.' {
                chars.next(); //skip dot
                return Ok(".".to_owned() + &read_digits(chars)?);
            }
            return Ok(String::new());
        }
    }
}

fn next_char(chars: &mut Peekable<CharIndices>) -> Option<char> {
    match chars.peek() {
        None => return None,
        Some(el) => {
            let (_, ch) = el;
            return Some(*ch);
        }
    }
}

fn consume_spaces(chars: &mut Peekable<CharIndices>) {
    loop {
        match next_char(chars) {
            None => return,
            Some(ch) => {
                if ch.is_whitespace() {
                    chars.next();
                }
            }
        }
    }
}

fn make_err(s: String) -> JSONParseError {
    JSONParseError { reason: s }
}

fn unexpected_eof() -> JSONParseError {
    make_err(ERROR_ENDED_UNEXPECTEDLY.to_owned())
}

fn unexpected_charachter(position: usize, ch: char) -> JSONParseError {
    make_err(format!(
        "Unexpected charachter {} at position {}",
        ch, position
    ))
}

fn invalid_escape_sequence(position: usize, s: &str) -> JSONParseError {
    make_err(format!(
        "Invalid escape sequence {} at position {}",
        s, position
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    fn assert_parse_str_err(query: &str) {
        parse_str(&mut query.char_indices().peekable())
            .expect_err(&format!("Invalid value {} parsed", query));
    }

    fn assert_parse_str(query: &str, res: &str) {
        let parsed = parse_str(&mut query.char_indices().peekable()).unwrap();
        if let JSONValue::JSONString(s) = parsed {
            assert_eq!(s, res);
            return;
        }
        panic!(format!("Invalid parse result type {:?}", parsed));
    }

    #[test]
    fn test_valid_string_examples() {
        assert_parse_str("\"asd\"", "asd");
        assert_parse_str("\"as asd  asd d\\\"\"", "as asd  asd d\"");
        assert_parse_str("\"asd\\r\\n\\t\"", "asd\r\n\t");
        assert_parse_str("\"\\u0041\"", "A");
        assert_parse_str("\"unicode sequence \\uc328\"", "unicode sequence 쌨");
    }

    #[test]
    fn test_invalid_string_examples() {
        assert_parse_str_err("no quotes");
        assert_parse_str_err("\"not_closed");
        assert_parse_str_err("not opened");
        assert_parse_str_err("\"invalid escape \\x \"");
    }

    #[test]
    fn valid_parse_bull() {
        for s in vec!["true", "true, ", "true  asdpjmklmo"] {
            match parse_true(&mut s.char_indices().peekable()).unwrap() {
                JSONValue::JSONBool(true) => {}
                _ => panic!("{} supposed to be true!", s),
            }
        }
        for s in vec!["false", "false, ", "false  asdpjmklmo"] {
            match parse_false(&mut s.char_indices().peekable()).unwrap() {
                JSONValue::JSONBool(false) => {}
                _ => panic!("{} supposed to be false!", s),
            }
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
            match parse_num(&mut s.0.char_indices().peekable()).unwrap() {
                JSONValue::JSONNumber(f) => assert_eq!(f, s.1),
                _ => panic!("Invalid value type while parsing {}", s.0),
            }
        }
    }

    #[test]
    fn test_invalid_parse_num() {
        for s in vec![
            "a123",
            "00.123",
            "1234a4",
            "+123",
            "a1u2djasjda",
            "123е123", //cicyllic "e"
            "123.0Ee123123123",
            "123+E123",
        ] {
            println!("Checking {}", s);
            parse_num(&mut s.char_indices().peekable())
                .expect_err(&format!("Expected to fail while parsing {}", s));
        }
    }

    #[test]
    fn test_valid_parse_null() {
        for s in vec!["null", "null, ", "null ", "null!"] {
            match parse_null(&mut s.char_indices().peekable()).unwrap() {
                JSONValue::JSONNull() => {}
                _ => panic!("Invalid result type for null parse!"),
            }
        }
    }

    #[test]
    fn invalid_parse_null() {
        for s in vec!["NULL", "!null", "asd", "><>OP"] {
            parse_null(&mut s.char_indices().peekable())
                .expect_err(&format!("Should not be parsed as null! {}", s));
        }
    }

}
