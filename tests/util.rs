#[macro_use]
extern crate serde_json;

extern crate task_diff;
use task_diff::util;

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
        assert_eq!(util::parse_environment(&env).unwrap(), result);
    }
}
