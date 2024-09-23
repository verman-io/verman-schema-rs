use super::*;
use crate::models::CommonContent;

#[test]
fn interpolate_test() {
    let common_output = interpolate(&CommonContent {
        content: Some(serde_json::Value::String(String::from("$A"))),
        env: Some(indexmap::indexmap! {
            String::from("A") => serde_json::Value::String(String::from("$C")),
            String::from("B") => serde_json::Value::String(String::from("$D")),
            String::from("C") => serde_json::Value::String(String::from("$B")),
            String::from("D") => serde_json::Value::String(String::from("goal")),
        }),
    })
    .unwrap();
    assert_eq!(
        common_output.content.unwrap().as_str().unwrap(),
        String::from("goal")
    );
}
