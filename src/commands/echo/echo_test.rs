use super::*;
use crate::models::CommonContent;

#[test]
fn echo_simple_test() {
    let content_vec_u8 = serde_json::Value::String(String::from("Hello"));
    let b = echo(&CommonContent {
        content: Some(content_vec_u8.clone()),
        env: None,
    })
    .unwrap();
    assert_eq!(b.content, Some(content_vec_u8));
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
    assert_eq!(b.content, Some("Hello ${weird} var".as_bytes().to_vec()));
}
