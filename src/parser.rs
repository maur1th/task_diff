use std::io::{self, Error, ErrorKind};

use serde_json::{self, Map, Value};

use util::{parse_environment, wrap, zip_to_end, Line};

//
// Entrypoint function
//

pub fn diff(a: &Value, b: &Value) -> io::Result<Vec<Line>> {
    match (a, b) {
        (Value::Object(a), Value::Object(b)) => {
            let mut a2 = a.clone();
            let mut b2 = b.clone();
            let env_diff = diff_env(a2.remove("environment"), b2.remove("environment"));
            let diff = diff_obj(&a2, &b2);
            Ok(diff.into_iter().chain(env_diff.into_iter()).collect())
        }
        (Value::Array(a), Value::Array(b)) => Ok(diff_array(a.clone(), b.clone())),
        _ => Err(Error::new(
            ErrorKind::InvalidInput,
            "Different types cannot be compared",
        )),
    }
}

//
// Helper functions
//

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
        }).filter(|l| l.diff != 'x')
        .collect();
    for (key, val) in new_b.iter() {
        result.push(Line::new('+', format!(r#""{}": {}"#, key, val)))
    }
    result.sort_unstable_by(|a, b| a.contents.cmp(&b.contents));
    result
}

fn diff_array(a: Vec<Value>, b: Vec<Value>) -> Vec<Line> {
    if a.iter().chain(b.iter()).all(|v| v.is_object()) {
        let empty_map = json!({});
        let zip = zip_to_end(a, b);
        zip.iter()
            .flat_map(|(a, b)| match (a, b) {
                (Some(a), Some(b)) => wrap(diff(a, b).unwrap(), ""),
                (Some(a), None) => wrap(diff(a, &empty_map).unwrap(), ""),
                (None, Some(b)) => wrap(diff(&empty_map, b).unwrap(), ""),
                _ => vec![],
            }).collect()
    } else {
        let to_json = |t| serde_json::to_value(t).unwrap();
        vec![Line::new(
            '~',
            format!(r#"{} => {}"#, to_json(a), to_json(b)),
        )]
    }
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
