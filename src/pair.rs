use std::io::{self, Error, ErrorKind::InvalidInput};

use serde_json::{self, Map, Value};

pub struct Pair {
    pub a: Value,
    pub b: Value,
}

impl Pair {
    fn clean_string(s: &str) -> io::Result<Value> {
        let result = s.trim().trim_matches('"').to_owned();
        let value = serde_json::from_str(&result)?;
        replace_in_tree("environment", value, parse_environment)
            .ok_or(Error::new(InvalidInput, "Invalid input"))
    }

    fn trim(input: &str) -> Option<&str> {
        let start = input.find('"')?;
        let end = input.rfind('"')?;
        Some(&input[start..end])
    }

    pub fn new(input: &str) -> io::Result<Pair> {
        let error = "Invalid input";
        let trimmed = Pair::trim(input).ok_or(Error::new(InvalidInput, error))?;
        let separator = " => ";
        let index = trimmed
            .find(separator)
            .ok_or(Error::new(InvalidInput, error))?;
        let a = Pair::clean_string(&trimmed[..index])?;
        let b = Pair::clean_string(&trimmed[index + separator.len()..])?;
        Ok(Pair { a, b })
    }
}

pub fn replace_in_tree<F>(key: &str, tree: Value, mut f: F) -> Option<Value>
where
    F: FnMut(&Value) -> Option<Value> + Copy,
{
    match tree {
        Value::Object(object) => {
            let mut new_object = object.clone();
            if let Some(value) = object.get(key) {
                new_object.insert(key.to_owned(), f(value)?);
            }
            Some(Value::Object(new_object))
        }
        Value::Array(array) => {
            let mut new_array: Vec<Value> = vec![];
            for item in array {
                new_array.push(replace_in_tree(key, item, f)?);
            }
            Some(Value::Array(new_array))
        }
        _ => Some(tree),
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
