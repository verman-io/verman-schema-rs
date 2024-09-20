use std::io::Write;

use super::*;
use crate::models::CommonContent;

const MOCK_FILTERS: [(&'static str, &'static str); 3] =
    [(".[0]", "null"), (".[1]", "[true,null]"), (".[2]", "5")];

lazy_static::lazy_static! {
    static ref MOCK_DATA_SERDE_JSON: serde_json::Value =
        serde_json::json!("[null, [true, null], 5]");
}

#[test]
fn test_jaq_runner0() {
    let mock_data_jaq_json: Vec<jaq_json::Val> = vec![
        jaq_json::Val::Null,
        jaq_json::Val::Arr(std::rc::Rc::new(vec![
            jaq_json::Val::Bool(true),
            jaq_json::Val::Null,
        ])),
        jaq_json::Val::Int(5),
    ];
    MOCK_FILTERS.iter().for_each(|(input, expect)| {
        let inputs = Box::new(std::iter::once(Ok(jaq_json::Val::Arr(std::rc::Rc::new(
            mock_data_jaq_json.clone(),
        )))));

        let (vars, filter) = jaq_utils::vars_filter_from_code(input).unwrap();

        let mut buf = Vec::<u8>::new();
        let _result: bool = jaq_runner(&filter, vars.clone(), false, inputs, |v| {
            buf.write_all(v.to_string().as_bytes())
        })
        .unwrap()
        .unwrap();
        /*assert!(_result);*/
        assert_eq!(std::str::from_utf8(buf.as_slice()).unwrap(), *expect);
    });
}

/*#[test]
fn test_jaq_runner1() {
    let mock_data_serde_json: serde_json::Value = MOCK_DATA_SERDE_JSON.clone();

    MOCK_FILTERS.iter().for_each(|(input, expect)| {
        let inputs = Box::new(std::iter::once(Ok(jaq_json::Val::from(mock_data_serde_json.clone()))));

        let (vars, filter) = jaq_utils::vars_filter_from_code(input).unwrap();

        let mut buf = Vec::<u8>::new();
        let _result: bool = jaq_runner(&filter, vars.clone(), false, inputs, |v| {
            buf.write_all(v.to_string().as_bytes())
        })
        .unwrap()
        .unwrap();
        /*assert!(_result);*/
        assert_eq!(std::str::from_utf8(buf.as_slice()).unwrap(), *expect);
    });
} */

#[test]
fn test_jaq() {
    let input_common = CommonContent {
        content: Some(serde_json::Value::String(String::from(".[1]"))),
        env: Some(indexmap::indexmap! {
            String::from("PREVIOUS_TASK_CONTENT") => serde_json::json!([1,{"stuff": true}])
        }),
    };
    let result_common = jaq(&input_common).unwrap();
    assert_eq!(
        result_common.content.unwrap(),
        serde_json::Value::String(String::from("{\"stuff\":true}"))
    );
}
