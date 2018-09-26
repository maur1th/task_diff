use serde_json::Value;

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
