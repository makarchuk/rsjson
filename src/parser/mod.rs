use super::*;
use std::char;
use std::iter::Peekable;
use std::str::CharIndices;

#[cfg(test)]
mod tests;

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
const DOT: char = '.';
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
    match chars.next() {
        None => return Ok(val),
        Some(el) => {
            let (i, ch) = el;
            return Err(unexpected_character(i, ch));
        }
    }
}

pub fn parse_value(chars: &mut Peekable<CharIndices>) -> Result<JSONValue, JSONParseError> {
    match next_char(chars) {
        None => return Err(make_err("Empty string provided".to_owned())),
        Some(ch) => match ch {
            OBJECT_START => return Ok(JSONValue::JSONObject(parse_object(chars)?)),
            QUOTE => return Ok(JSONValue::JSONString(parse_str(chars)?)),
            TRUE_START => return Ok(JSONValue::JSONBool(parse_true(chars)?)),
            FALSE_START => return Ok(JSONValue::JSONBool(parse_false(chars)?)),
            NULL_START => {
                parse_null(chars)?;
                return Ok(JSONValue::JSONNull());
            }
            MINUS => return Ok(JSONValue::JSONNumber(parse_num(chars)?)),
            '0'...'9' => return Ok(JSONValue::JSONNumber(parse_num(chars)?)),
            ARRAY_START => return Ok(JSONValue::JSONArray(parse_array(chars)?)),
            _ => {
                let (i, ch) = chars.next().unwrap();
                return Err(unexpected_character(i, ch));
            }
        },
    };
}

fn parse_array(chars: &mut Peekable<CharIndices>) -> Result<Vec<Box<JSONValue>>, JSONParseError> {
    let mut result: Vec<Box<JSONValue>> = vec![];
    read_known_char(chars, ARRAY_START)?;
    consume_spaces(chars);
    match next_char(chars).ok_or(unexpected_eof())? {
        ARRAY_END => {
            chars.next();
            return Ok(result);
        }
        _ => (),
    }
    loop {
        consume_spaces(chars);
        result.push(Box::new(parse_value(chars)?));
        consume_spaces(chars);
        let (i, ch) = chars.next().ok_or(unexpected_eof())?;
        match ch {
            ARRAY_END => return Ok(result),
            COMMA => (),
            _ => {
                return Err(unexpected_character(i, ch));
            }
        }
    }
}

fn parse_object(
    chars: &mut Peekable<CharIndices>,
) -> Result<HashMap<String, Box<JSONValue>>, JSONParseError> {
    let mut result: HashMap<String, Box<JSONValue>> = HashMap::new();
    read_known_char(chars, OBJECT_START)?;
    match next_char(chars).ok_or(unexpected_eof())? {
        OBJECT_END => {
            chars.next();
            return Ok(result);
        }
        _ => (),
    }
    loop {
        consume_spaces(chars);
        let key = parse_str(chars)?;
        consume_spaces(chars);
        read_known_char(chars, COLON)?;
        consume_spaces(chars);
        let value = parse_value(chars)?;
        result.insert(key, Box::new(value));
        consume_spaces(chars);
        let (i, ch) = chars.next().ok_or(unexpected_eof())?;
        match ch {
            OBJECT_END => return Ok(result),
            COMMA => (),
            _ => return Err(unexpected_character(i, ch)),
        }
    }
}

fn parse_const<T>(
    chars: &mut Peekable<CharIndices>,
    str_value: &str,
    value: T,
) -> Result<T, JSONParseError> {
    for correct_char in str_value.chars() {
        let (i, ch) = chars.next().ok_or(unexpected_eof())?;
        if correct_char != ch {
            return Err(unexpected_character(i, ch));
        }
    }
    return Ok(value);
}

fn parse_true(chars: &mut Peekable<CharIndices>) -> Result<bool, JSONParseError> {
    return parse_const(chars, BOOL_TRUE, true);
}

fn parse_false(chars: &mut Peekable<CharIndices>) -> Result<bool, JSONParseError> {
    return parse_const(chars, BOOL_FALSE, false);
}

fn parse_null(chars: &mut Peekable<CharIndices>) -> Result<(), JSONParseError> {
    return parse_const(chars, NULL, ());
}

fn parse_str(chars: &mut Peekable<CharIndices>) -> Result<String, JSONParseError> {
    let mut result = String::new();
    read_known_char(chars, QUOTE)?;
    loop {
        let (i, ch) = chars.next().ok_or(unexpected_eof())?;
        match ch {
            QUOTE => return Ok(result),
            ESCAPE => result.push_str(&read_escape_char(chars)?),
            '\0'...'\x1F' => return Err(unexpected_character(i, ch)),
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

fn parse_num(chars: &mut Peekable<CharIndices>) -> Result<f64, JSONParseError> {
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
        '1'...'9' => {
            num.push_str(&read_digits(chars)?);
        }
        _ => {
            let (i, ch) = chars.next().ok_or(unexpected_eof())?;
            return Err(unexpected_character(i, ch));
        }
    }
    num.push_str(&read_fraction(chars)?);
    match next_char(chars) {
        None => (),
        Some(ch) => {
            if ch == 'e' || ch == 'E' {
                chars.next().unwrap();
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
                num.push_str(&read_digits(chars)?);
            }
        }
    }
    match num.parse() {
        Ok(n) => return Ok(n),
        Err(_) => return Err(make_err(format!("Unable to parse number {}", num))),
    }
}

fn read_digits(chars: &mut Peekable<CharIndices>) -> Result<String, JSONParseError> {
    let mut result = String::new();
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

//Read optional fraction part. It can be empty, but it can't start with number!
fn read_fraction(chars: &mut Peekable<CharIndices>) -> Result<String, JSONParseError> {
    match next_char(chars) {
        None => return Ok(String::new()),
        Some(ch) => {
            match ch {
                DOT => {
                    chars.next(); //skip dot
                    let digits = &read_digits(chars)?;
                    if digits.len() == 0 {
                        let (i, ch) = chars.next().ok_or(unexpected_eof())?;
                        return Err(unexpected_character(i, ch));
                    }
                    return Ok(".".to_owned() + digits);
                }
                '0'...'9' => {
                    let (i, ch) = chars.next().unwrap();
                    return Err(unexpected_character(i, ch));
                }
                _ => return Ok(String::new()),
            }
        }
    }
}

fn read_known_char(
    chars: &mut Peekable<CharIndices>,
    expected: char,
) -> Result<(), JSONParseError> {
    let (i, ch) = chars.next().ok_or(unexpected_eof())?;
    if ch != expected {
        return Err(make_err(format!(
            "Unexpected charachter {} at position {}. Expected {}",
            ch, i, expected
        )));
    };
    return Ok(());
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
                if is_whitespace(ch) {
                    chars.next();
                } else {
                    return;
                }
            }
        }
    }
}

fn is_whitespace(ch: char) -> bool {
    match ch {
        '\x09' | '\x0a' | '\x0d' | '\x20' => true,
        _ => false,
    }
}

fn make_err(s: String) -> JSONParseError {
    JSONParseError { reason: s }
}

fn unexpected_eof() -> JSONParseError {
    make_err(ERROR_ENDED_UNEXPECTEDLY.to_owned())
}

fn unexpected_character(position: usize, ch: char) -> JSONParseError {
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
