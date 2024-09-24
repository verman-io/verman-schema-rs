use super::*;
use crate::models::CommonContent;

#[test]
fn env_simple_test() {
    let content_val = Some(serde_json::Value::String(String::from("Hello")));
    let b = env(&CommonContent {
        content: content_val.clone(),
        env: None,
    })
    .unwrap();
    assert_eq!(b.content, content_val);
}
