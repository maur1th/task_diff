#[macro_use]
extern crate serde_json;

extern crate task_diff;
use task_diff::parser;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diff_obj_delete() {
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
        let mut result: Vec<String> = parser::diff(&a, &b)
            .unwrap()
            .iter()
            .map(|l| format!("{}", l))
            .collect();
        let mut expected = vec![
            r#"- "baz": "removed""#,
            r#"+ "quux": "added""#,
            r#"~ "qux": "will change" => "changed!""#,
        ];
        result.sort_unstable();
        expected.sort_unstable();
        assert_eq!(result, expected);
    }

    #[test]
    fn diff_array_obj() {
        let a = json!([
            {"foo": "same", "baz": "removed"},
            {"qux": "will change"},
        ]);
        let b = json!([
            {"foo": "same"},
            {"quux": "added", "qux": "changed!"},
        ]);
        let mut result: Vec<String> = parser::diff(&a, &b)
            .unwrap()
            .iter()
            .map(|l| format!("{}", l))
            .collect();
        let mut expected = vec![
            r#"{"#,
            r#"  - "baz": "removed""#,
            r#"}"#,
            r#"{"#,
            r#"  + "quux": "added""#,
            r#"  ~ "qux": "will change" => "changed!""#,
            r#"}"#,
        ];
        result.sort_unstable();
        expected.sort_unstable();
        assert_eq!(result, expected);
    }

    #[test]
    fn diff_array_obj_removed() {
        let a = json!([
            {"foo": "same"},
            {"baz": "removed"},
        ]);
        let b = json!([
            {"foo": "same"},
        ]);
        let result: Vec<String> = parser::diff(&a, &b)
            .unwrap()
            .iter()
            .map(|l| format!("{}", l))
            .collect();
        let expected = vec![r#"{"#, r#"  - "baz": "removed""#, r#"}"#];
        assert_eq!(result, expected);
    }

    #[test]
    fn diff_array() {
        let a = json!([1, 2]);
        let b = json!([1, 2, 3]);
        let result: Vec<String> = parser::diff(&a, &b)
            .unwrap()
            .iter()
            .map(|l| format!("{}", l))
            .collect();
        let expected = vec![r#"~ [1,2] => [1,2,3]"#];
        assert_eq!(result, expected);
    }

    #[test]
    fn diff_array_vs_obj() {
        let a = json!({});
        let b = json!([]);
        assert!(parser::diff(&a, &b).is_err());
    }

    #[test]
    fn diff_obj_empty() {
        let a = json!({});
        let b = json!({});
        assert!(parser::diff(&a, &b).unwrap().is_empty());
    }

    #[test]
    fn diff_array_empty() {
        let a = json!([]);
        let b = json!([]);
        assert!(parser::diff(&a, &b).unwrap().is_empty());
    }

    #[test]
    fn diff_env() {
        let a = json!({
            "environment": {
                "foo": "kept",
                "bar": "removed",
                "qux": "will change",
            }
        });
        let b = json!({
            "environment": {
                "foo": "kept",
                "baz": "added",
                "qux": "changed!",
            }
        });
        let mut result: Vec<String> = parser::diff(&a, &b)
            .unwrap()
            .iter()
            .map(|l| format!("{}", l))
            .collect();
        let mut expected = vec![
            r#""environment": {"#,
            r#"  - "bar": "removed""#,
            r#"  + "baz": "added""#,
            r#"  ~ "qux": "will change" => "changed!""#,
            r#"}"#,
        ];
        result.sort_unstable();
        expected.sort_unstable();
        assert_eq!(result, expected);
    }
}
