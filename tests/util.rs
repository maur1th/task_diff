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

    #[test]
    fn replace_in_object_tree() {
        let env = json!({
            "environment": [
                {"name": "foo", "value": "bar"},
                {"name": "baz", "value": "qux"},
            ]
        });
        let result = json!({
            "environment": {
                "foo": "bar",
                "baz": "qux",
            }
        });
        assert_eq!(
            util::replace_in_tree("environment", env, util::parse_environment).unwrap(),
            result
        );
    }

    #[test]
    fn replace_in_array_tree() {
        let env = json!([{
            "environment": [
                {"name": "foo", "value": "bar"},
                {"name": "baz", "value": "qux"},
            ]
        }]);
        let result = json!([{
            "environment": {
                "foo": "bar",
                "baz": "qux",
            }
        }]);
        assert_eq!(
            util::replace_in_tree("environment", env, util::parse_environment).unwrap(),
            result
        );
    }
}
