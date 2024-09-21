use super::*;
use crate::models::CommonContent;

#[test]
fn env_simple_test() {
    let content_vec_u8 = serde_json::Value::String(String::from("Hello"));
    let b = env(&CommonContent {
        content: Some(content_vec_u8.clone()),
        env: None,
    })
    .unwrap();
    assert_eq!(b.content, Some(content_vec_u8));
}
