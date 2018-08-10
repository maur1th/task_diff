use std::fmt;
use std::io::{self, Error, ErrorKind};

use serde_json::{self, Map, Value};

pub struct Pair {
    pub a: Value,
    pub b: Value,
}

impl Pair {
    fn clean_string(s: &str) -> Result<Value, serde_json::Error> {
        let mut result = s.trim().trim_matches('"').to_owned();
        result.retain(|c| c != '\\');
        serde_json::from_str(&result)
    }

    pub fn new(s: &str) -> io::Result<Pair> {
        let separator = " => ";
        let index = s
            .find(separator)
            .ok_or(Error::new(ErrorKind::InvalidInput, "Wrong input"))?;
        let a = Pair::clean_string(&s[..index])?;
        let b = Pair::clean_string(&s[index + separator.len()..])?;
        Ok(Pair { a, b })
    }
}

pub struct Line {
    pub diff: char,
    pub depth: usize,
    pub contents: String,
}

impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let diff = match self.diff {
            '.' => "".to_owned(),
            _ => format!("{} ", self.diff),
        };
        write!(f, "{}{}{}", " ".repeat(self.depth * 2), diff, self.contents)
    }
}

impl Line {
    pub fn new(diff: char, contents: String) -> Line {
        Line {
            diff,
            depth: 0,
            contents,
        }
    }
}

pub fn parse_environment(env: &Value) -> Option<Value> {
    let mut result: Map<String, Value> = Map::new();
    for t in env.as_array()?.iter() {
        let map = t.as_object()?;
        result.insert(
            map.get("name")?.as_str()?.to_owned(),
            map.get("value")?.to_owned(),
        );
    }
    return Some(json!(&result));
}

pub fn wrap(lines: Vec<Line>, name: &str) -> Vec<Line> {
    let mut result: Vec<Line> = vec![];
    if !lines.is_empty() {
        result.push(Line::new('.', format!("{}{{", name)));
        for mut line in lines {
            line.depth += 1;
            result.push(line);
        }
        result.push(Line::new('.', "}".to_owned()));
    }
    result
}

pub fn zip_to_end(a: Vec<Value>, b: Vec<Value>) -> Vec<(Option<Value>, Option<Value>)> {
    let mut result = Vec::<(Option<Value>, Option<Value>)>::new();
    let mut iter_a = a.into_iter();
    let mut iter_b = b.into_iter();
    loop {
        let next_a = iter_a.next();
        let next_b = iter_b.next();
        if next_a.is_none() && next_b.is_none() {
            break;
        }
        result.push((next_a, next_b));
    }
    result
}

//
// Tests
//

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_environment_array() {
        let env = json!([
            {"name": "foo", "value": "bar"},
            {"name": "baz", "value": "qux"},
        ]);
        let result = json!({
            "foo": "bar",
            "baz": "qux",
        });
        assert_eq!(parse_environment(&env).unwrap(), result);
    }
}
