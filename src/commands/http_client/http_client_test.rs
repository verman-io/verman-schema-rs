use crate::commands::http_client::http;
use crate::models::{CommonContent, Expectation, HttpArgs, HttpCommandArgs};
use crate::test_models::{Message, HTTPBIN_URL};

#[tokio::test]
async fn test_httpbin_post_empty_body() {
    let result = http(&HttpCommandArgs {
        args: HttpArgs {
            url: format!("{}/post", HTTPBIN_URL)
                .parse::<http::uri::Uri>()
                .unwrap(),
            method: http::method::Method::POST,
            headers: None,
        },
        common_content: CommonContent::default(),
        expectation: Expectation::default(),
    })
    .await
    .unwrap()
    .1
    .env
    .unwrap();
    let out = result.get("PREVIOUS_TASK_CONTENT").unwrap();
    let httpbin_post_response: crate::test_models::HttpBinPostResponse =
        serde_json::from_str(out.to_owned().left().unwrap().as_str()).unwrap();
    assert_eq!(httpbin_post_response.json, serde_json::Value::Null);
}

#[tokio::test]
async fn test_httpbin_post_message_body() {
    let result = http(&HttpCommandArgs {
        args: HttpArgs {
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
        common_content: CommonContent{
            content: Some("{\"message\": \"greetings\"}".as_bytes().to_vec()),
            // ^ could also construct `Message` and `serde_json` it down, like in next test
            ..CommonContent::default()
        },
        expectation: Expectation::default(),
    })
        .await.unwrap().1.env.unwrap();
    let out = result.get("PREVIOUS_TASK_CONTENT").unwrap();
    let httpbin_post_response: crate::test_models::HttpBinPostResponse =
        serde_json::from_str(out.to_owned().left().unwrap().as_str()).unwrap();
    let message: Message = serde_json::from_value(httpbin_post_response.json).unwrap();
    assert_eq!(message.message, "greetings");
}

#[tokio::test]
async fn test_httpbin_post_message_body_and_env_vars() {
    let message_input = Message {
        message: String::from("greetings to ${ME}"),
    };

    let result = http(&HttpCommandArgs {
        args: HttpArgs {
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
        common_content: CommonContent {
            content: Some(serde_json::to_string(&message_input).unwrap().into_bytes()),
            // ^ could also construct `Message` and `serde_json` it down
            env: Some(indexmap::indexmap! {
                    String::from("ME") => either::Left(String::from("Prine"))
                }),
            ..CommonContent::default()
        },
        expectation: Expectation::default(),
    })
        .await.unwrap().1.env.unwrap();
    let out = result.get("PREVIOUS_TASK_CONTENT").unwrap();
    let httpbin_post_response: crate::test_models::HttpBinPostResponse =
        serde_json::from_str(out.to_owned().left().unwrap().as_str()).unwrap();
    let message: Message = serde_json::from_value(httpbin_post_response.json).unwrap();
    assert_eq!(message.message, "greetings to Prine");
}
