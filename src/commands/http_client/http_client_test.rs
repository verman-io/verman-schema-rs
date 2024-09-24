use crate::commands::command::CommandKey;
use crate::commands::http_client::http;
use crate::models::{CommonContent, Expectation, HttpArgs, HttpCommandArgs};
use crate::test_models::{Message, HTTPBIN_URL};

#[tokio::test]
async fn test_httpbin_post_empty_body() {
    let result = http(&HttpCommandArgs::new(
        HttpArgs {
            url: format!("{}/post", HTTPBIN_URL)
                .parse::<http::uri::Uri>()
                .unwrap(),
            method: http::method::Method::POST,
            headers: None,
        },
        CommonContent::default(),
        Expectation::default(),
    ))
    .await
    .unwrap()
    .1
    .env
    .unwrap();
    let previous_task_content = result
        .get(CommandKey::PreviousContent.to_string().as_str())
        .unwrap();
    let httpbin_post_response: crate::test_models::HttpBinPostResponse =
        serde_json::from_value(previous_task_content.to_owned()).unwrap();
    assert_eq!(httpbin_post_response.json, serde_json::Value::Null);
}

#[tokio::test]
async fn test_httpbin_post_message_body() {
    let result = http(&HttpCommandArgs::new(
        HttpArgs {
            url: format!("{}/post", HTTPBIN_URL)
                .parse::<http::uri::Uri>()
                .unwrap(),
            method: http::method::Method::POST,
            headers: Some(vec![
                indexmap::indexmap! {
                        String::from("Content-Type") => serde_json_extensions::ValueNoObjOrArr::String(String::from("application/json")),
                    }
            ])
        },
        CommonContent{
            content: Some(serde_json::json!({"message": "greetings"})),
            ..CommonContent::default()
        },
        Expectation::default(),
    ))
        .await.unwrap().1.env.unwrap();
    assert!(result.contains_key(CommandKey::PreviousContent.to_string().as_str()));
    let previous_task_content = result
        .get(CommandKey::PreviousContent.to_string().as_str())
        .unwrap();
    let httpbin_post_response: crate::test_models::HttpBinPostResponse<Message> =
        serde_json::from_value(previous_task_content.to_owned()).unwrap();
    let message0: Message = serde_json::from_str(httpbin_post_response.data.as_str()).unwrap();
    assert_eq!(message0.message, "greetings");
    let message1: Message = httpbin_post_response.json;
    assert_eq!(message1.message, "greetings");
}

#[tokio::test]
async fn test_httpbin_post_message_body_and_env_vars() {
    let message_input = Message {
        message: String::from("greetings to ${ME}"),
    };

    let result = http(&HttpCommandArgs::new(
        HttpArgs {
            url: format!("{}/post", HTTPBIN_URL)
                .parse::<http::uri::Uri>()
                .unwrap(),
            method: http::method::Method::POST,
            headers: Some(vec![
                indexmap::indexmap! {
                        String::from("Content-Type") => serde_json_extensions::ValueNoObjOrArr::String(String::from("application/json")),
                    }
            ])
        },
        CommonContent {
            content: Some(serde_json::to_value(&message_input).unwrap()),
            // ^ could also construct `Message` and `serde_json` it down
            env: Some(indexmap::indexmap! {
                    String::from("ME") => serde_json::Value::String(String::from("Prine"))
                }),
            ..CommonContent::default()
        },
        Expectation::default(),
    ))
        .await.unwrap().1.env.unwrap();

    let previous_task_content = result
        .get(CommandKey::PreviousContent.to_string().as_str())
        .unwrap();
    let httpbin_post_response: crate::test_models::HttpBinPostResponse =
        serde_json::from_value(previous_task_content.to_owned()).unwrap();
    let message: Message = serde_json::from_value(httpbin_post_response.json).unwrap();
    assert_eq!(message.message, "greetings to Prine");
}
