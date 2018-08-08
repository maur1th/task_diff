use std::fmt;

#[macro_use]
extern crate serde_json;
use serde_json::{Map, Value};

struct Line {
    diff: char,
    contents: String,
}

impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.diff, self.contents)
    }
}

impl Line {
    pub fn new(diff: char, contents: String) -> Line {
        Line { diff, contents }
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

fn diff(t: &Value, u: &Value) -> Option<Vec<Line>> {
    let mut new_u = u.clone();
    let u_obj = new_u.as_object_mut()?;
    let mut result: Vec<Line> = t.as_object()?
        .into_iter()
        .map(|(key, val)| {
            if let Some(u_val) = u_obj.remove(key) {
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
    for (key, val) in u_obj.iter() {
        result.push(Line::new('+', format!(r#""{}": {}"#, key, val)))
    }
    result.sort_unstable_by(|a, b| a.contents.cmp(&b.contents));
    Some(result)
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
    fn test_diff_map_delete() {
        let t = json!({
            "foo": "same",
            "baz": "removed",
            "qux": "will change",
        });
        let u = json!({
            "foo": "same",
            "quux": "added",
            "qux": "changed!",
        });
        let result: Vec<String> = diff(&t, &u)
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
}
