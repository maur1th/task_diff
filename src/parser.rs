use std::io::{self, Error, ErrorKind};

use serde_json::{self, Map, Value};

use crate::util::zip_to_end;

use crate::line::{wrap, Line, WrapperKind};

//
// Entrypoint function
//

pub fn diff(a: &Value, b: &Value) -> io::Result<Vec<Line>> {
    match (a, b) {
        (Value::Object(a), Value::Object(b)) => Ok(diff_obj(&a, &b)),
        (Value::Array(a), Value::Array(b)) => Ok(diff_array(&a, &b)),
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
    let mut b2 = b.clone();
    let mut result: Vec<Line> = a
        .iter()
        .flat_map(|(key, val)| match b2.remove(key) {
            Some(ref bval) if val == bval => vec![Line::new('x', String::new())],
            Some(ref bval) if val.is_object() && bval.is_object() => wrap(
                diff(val, &bval).expect("Invalid input"),
                &format!("\"{}\": ", key),
                WrapperKind::Object,
            ),
            Some(ref bval) if val.is_array() && bval.is_array() => wrap(
                diff(val, &bval).expect("Invalid input"),
                &format!("\"{}\": ", key),
                WrapperKind::Array,
            ),
            Some(bval) => vec![Line::new('~', format!(r#""{}": {} => {}"#, key, val, bval))],
            _ => vec![Line::new('-', format!(r#""{}": {}"#, key, val))],
        }).filter(|l| l.diff != 'x')
        .collect();
    for (key, val) in b2.iter() {
        result.push(Line::new('+', format!(r#""{}": {}"#, key, val)))
    }
    result
}

fn diff_array(a: &[Value], b: &[Value]) -> Vec<Line> {
    if a.iter().chain(b.iter()).all(|v| v.is_object()) {
        let empty_map = json!({});
        let zip = zip_to_end(a.to_vec(), b.to_vec());
        zip.iter()
            .flat_map(|(a, b)| match (a, b) {
                (Some(a), Some(b)) => wrap(diff(a, b).unwrap(), "", WrapperKind::Object),
                (Some(a), None) => wrap(diff(a, &empty_map).unwrap(), "", WrapperKind::Object),
                (None, Some(b)) => wrap(diff(&empty_map, b).unwrap(), "", WrapperKind::Object),
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
