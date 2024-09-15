use super::*;
use crate::models::CommonContent;

#[test]
fn echo_simple_test() {
    let content_vec_u8 = "Hello".as_bytes().to_owned();
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
        content: Some("Hello ${VAR} var".as_bytes().to_owned()),
        env: Some(indexmap::indexmap! {
            String::from("VAR") => either::Left(String::from("${weird}"))
        }),
    })
    .unwrap();
    assert_eq!(b.content, Some("Hello ${weird} var".as_bytes().to_vec()));
}
