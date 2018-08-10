use std::fmt;
use std::io::{self, Error, ErrorKind};

#[macro_use]
extern crate serde_json;
use serde_json::{Map, Value};

pub struct Pair {
    pub a: Value,
    pub b: Value,
}

impl Pair {
    fn clean_string(s: &str) -> Result<Value, serde_json::Error> {
        // let pattern: &[_] = &['"', '[', ']'];
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
    diff: char,
    depth: usize,
    contents: String,
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

fn parse_environment(env: &Value) -> Option<Value> {
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

fn diff_obj(a: &Map<String, Value>, b: &Map<String, Value>) -> Vec<Line> {
    let mut new_b = b.clone();
    let mut result: Vec<Line> = a
        .iter()
        .map(|(key, val)| {
            if let Some(u_val) = new_b.remove(key) {
                if val == &u_val {
                    Line::new('x', String::new())
                } else {
                    Line::new('~', format!(r#""{}": {} => {}"#, key, val, u_val))
                }
            } else {
                Line::new('-', format!(r#""{}": {}"#, key, val))
            }
        })
        .filter(|l| l.diff != 'x')
        .collect();
    for (key, val) in new_b.iter() {
        result.push(Line::new('+', format!(r#""{}": {}"#, key, val)))
    }
    result.sort_unstable_by(|a, b| a.contents.cmp(&b.contents));
    result
}

fn wrap(lines: Vec<Line>, name: &str) -> Vec<Line> {
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

fn diff_obj_array(a: &[Value], b: &[Value]) -> Vec<Line> {
    let mut result: Vec<Line> = vec![];
    for (a, b) in a.iter().zip(b.iter()) {
        let lines = diff(a, b).unwrap();
        result.append(&mut wrap(lines, ""));
    }
    let delta: usize = ((a.len() - b.len()) as i32).abs() as usize;
    let empty_map: Map<String, Value> = Map::new();
    if a.len() < b.len() {
        result.extend(b[delta..].iter().flat_map(|v| {
            let lines = diff_obj(&empty_map, v.as_object().unwrap());
            wrap(lines, "")
        }));
    } else if a.len() > b.len() {
        result.extend(a[delta..].iter().flat_map(|v| {
            let lines = diff_obj(v.as_object().unwrap(), &empty_map);
            wrap(lines, "")
        }));
    }
    result
}

fn diff_array(a: &Vec<Value>, b: &Vec<Value>) -> Vec<Line> {
    let mut result: Vec<Line> = vec![];
    if a.iter().chain(b.iter()).all(|v| v.is_object()) {
        let lines = diff_obj_array(a, b);
        result.extend(lines);
    } else {
        let to_json = |t| serde_json::to_value(t).unwrap();
        result.push(Line::new(
            '~',
            format!(r#"{} => {}"#, to_json(a), to_json(b)),
        ))
    }
    result
}

fn diff_env(a: Option<Value>, b: Option<Value>) -> Vec<Line> {
    match (a, b) {
        (Some(a), Some(b)) => {
            let a_env = parse_environment(&a);
            let b_env = parse_environment(&b);
            let lines = diff_obj(
                a_env
                    .expect("Invalid environment value")
                    .as_object()
                    .unwrap(),
                b_env
                    .expect("Invalid environment value")
                    .as_object()
                    .unwrap(),
            );
            wrap(lines, "\"environment\": ")
        }
        _ => vec![],
    }
}

pub fn diff(a: &Value, b: &Value) -> io::Result<Vec<Line>> {
    match (a, b) {
        (Value::Object(a), Value::Object(b)) => {
            let mut a2 = a.clone();
            let mut b2 = b.clone();
            let env_diff = diff_env(a2.remove("environment"), b2.remove("environment"));
            let diff = diff_obj(&a2, &b2);
            Ok(diff.into_iter().chain(env_diff.into_iter()).collect())
        }
        (Value::Array(a), Value::Array(b)) => Ok(diff_array(&a, &b)),
        _ => Err(Error::new(
            ErrorKind::InvalidInput,
            "Different types cannot be compared",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_environment() {
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

    #[test]
    fn test_diff_obj_delete() {
        let a = json!({
            "foo": "same",
            "baz": "removed",
            "qux": "will change",
        });
        let b = json!({
            "foo": "same",
            "quux": "added",
            "qux": "changed!",
        });
        let result: Vec<String> = diff(&a, &b)
            .unwrap()
            .iter()
            .map(|l| format!("{}", l))
            .collect();
        let expected = vec![
            r#"- "baz": "removed""#,
            r#"+ "quux": "added""#,
            r#"~ "qux": "will change" => "changed!""#,
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_diff_array_obj() {
        let a = json!([
            {"foo": "same", "baz": "removed"},
            {"qux": "will change"},
        ]);
        let b = json!([
            {"foo": "same"},
            {"quux": "added", "qux": "changed!"},
        ]);
        let result: Vec<String> = diff(&a, &b)
            .unwrap()
            .iter()
            .map(|l| format!("{}", l))
            .collect();
        let expected = vec![
            r#"{"#,
            r#"  - "baz": "removed""#,
            r#"}"#,
            r#"{"#,
            r#"  + "quux": "added""#,
            r#"  ~ "qux": "will change" => "changed!""#,
            r#"}"#,
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_diff_array_obj_removed() {
        let a = json!([
            {"foo": "same"},
            {"baz": "removed"},
        ]);
        let b = json!([
            {"foo": "same"},
        ]);
        let result: Vec<String> = diff(&a, &b)
            .unwrap()
            .iter()
            .map(|l| format!("{}", l))
            .collect();
        let expected = vec![r#"{"#, r#"  - "baz": "removed""#, r#"}"#];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_diff_array() {
        let a = json!([1, 2]);
        let b = json!([1, 2, 3]);
        let result: Vec<String> = diff(&a, &b)
            .unwrap()
            .iter()
            .map(|l| format!("{}", l))
            .collect();
        let expected = vec![r#"~ [1,2] => [1,2,3]"#];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_diff_array_vs_obj() {
        let a = json!({});
        let b = json!([]);
        assert!(diff(&a, &b).is_err());
    }

    #[test]
    fn test_diff_obj_empty() {
        let a = json!({});
        let b = json!({});
        assert!(diff(&a, &b).unwrap().is_empty());
    }

    #[test]
    fn test_diff_array_empty() {
        let a = json!([]);
        let b = json!([]);
        assert!(diff(&a, &b).unwrap().is_empty());
    }

    #[test]
    fn test_diff_env() {
        let a = json!({
            "environment": [
                {"name": "foo", "value": "kept"},
                {"name": "bar", "value": "removed"},
                {"name": "qux", "value": "will change"},
            ]
        });
        let b = json!({
            "environment": [
                {"name": "foo", "value": "kept"},
                {"name": "baz", "value": "added"},
                {"name": "qux", "value": "changed!"},
            ]
        });
        let result: Vec<String> = diff(&a, &b)
            .unwrap()
            .iter()
            .map(|l| format!("{}", l))
            .collect();
        let expected = vec![
            r#""environment": {"#,
            r#"  - "bar": "removed""#,
            r#"  + "baz": "added""#,
            r#"  ~ "qux": "will change" => "changed!""#,
            r#"}"#,
        ];
        assert_eq!(result, expected);
    }
}
