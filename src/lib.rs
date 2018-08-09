use std::fmt;

#[macro_use]
extern crate serde_json;
use serde_json::{Map, Value};

struct Line {
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
    let mut result: Vec<Line> = a.iter()
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

fn wrap(lines: Vec<Line>) -> Vec<Line> {
    let mut result: Vec<Line> = vec![];
    if !lines.is_empty() {
        result.push(Line::new('.', "{".to_owned()));
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
        let lines = diff_obj(a.as_object().unwrap(), b.as_object().unwrap());
        result.append(&mut wrap(lines));
    }
    let delta: usize = ((a.len() - b.len()) as i32).abs() as usize;
    let empty_map: Map<String, Value> = Map::new();
    if a.len() < b.len() {
        result.extend(b[delta..].iter().flat_map(|v| {
            let lines = diff_obj(&empty_map, v.as_object().unwrap());
            wrap(lines)
        }));
    } else if a.len() > b.len() {
        result.extend(a[delta..].iter().flat_map(|v| {
            let lines = diff_obj(v.as_object().unwrap(), &empty_map);
            wrap(lines)
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

fn diff(a: &Value, b: &Value) -> Option<Vec<Line>> {
    match (&a, &b) {
        (Value::Object(a), Value::Object(b)) => Some(diff_obj(a, b)),
        (Value::Array(a), Value::Array(b)) => Some(diff_array(a, b)),
        _ => None,
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
        assert!(diff(&a, &b).is_none());
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
}
