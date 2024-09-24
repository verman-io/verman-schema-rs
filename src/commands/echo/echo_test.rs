use super::*;
use crate::models::CommonContent;

#[test]
fn echo_simple_test() {
    let content_val = Some(serde_json::Value::String(String::from("Hello")));
    let b = echo(&CommonContent {
        content: content_val.clone(),
        env: None,
    })
    .unwrap();
    assert_eq!(b.content, content_val);
}

#[test]
fn echo_env_test() {
    let b = echo(&CommonContent {
        content: Some(serde_json::Value::String(String::from("Hello ${VAR} var"))),
        env: Some(indexmap::indexmap! {
            String::from("VAR") => serde_json::Value::String(String::from("${weird}"))
        }),
    })
    .unwrap();
    assert_eq!(
        b.content,
        Some(serde_json::Value::String(String::from(
            "Hello ${weird} var"
        )))
    );
}
