use super::*;
use crate::models::CommonContent;

#[test]
fn set_env_test() {
    let common_input = CommonContent {
        content: None,
        env: Some(indexmap::indexmap! {
            String::from("ENV WAS SET") => either::Left(String::from("indeed"))
        }),
    };
    let common_output = set_env(&common_input).unwrap();
    assert_eq!(common_output, common_input);
}
