use crate::commands::CommandArgs;
use crate::models::{CommonContent, HttpArgs, HttpCommandArgs, Pipeline, Task};
use crate::test_models::{HttpBinPostResponse, Message, HTTPBIN_URL};

lazy_static::lazy_static! {
    static ref PIPELINE1: Pipeline = Pipeline {
        name: String::from(env!("CARGO_PKG_NAME")),
        version: String::from(env!("CARGO_PKG_VERSION")),
        description: String::from(env!("CARGO_PKG_DESCRIPTION")),
        url: String::from(env!("CARGO_PKG_REPOSITORY")),
        env: None,
        pipe: None,
        tasks: None,
        schemas: None,
        ..Pipeline::default()
    };
}

#[tokio::test]
async fn empty_pipeline_test() {
    let pipeline1: Pipeline = Pipeline {
        name: String::from(env!("CARGO_PKG_NAME")),
        version: String::from(env!("CARGO_PKG_VERSION")),
        description: String::from(env!("CARGO_PKG_DESCRIPTION")),
        url: String::from(env!("CARGO_PKG_REPOSITORY")),
        env: None,
        pipe: None,
        tasks: None,
        schemas: None,
        ..Pipeline::default()
    };
    let common = pipeline1.process().await.unwrap();
    assert_eq!(common, CommonContent::default());
}

#[tokio::test]
async fn empty_echo_task_empty_commands_pipeline_test() {
    let pipeline2: Pipeline = Pipeline {
        name: String::from(env!("CARGO_PKG_NAME")),
        version: String::from(env!("CARGO_PKG_VERSION")),
        description: String::from(env!("CARGO_PKG_DESCRIPTION")),
        url: String::from(env!("CARGO_PKG_REPOSITORY")),
        env: None,
        pipe: None,
        tasks: Some(indexmap::indexmap! {
        String::from("task0") =>
        Task {
            commands: vec![],
            input_schema: None,
            output_schema: None,
            env: None,
        }
        }),
        schemas: None,
        ..Pipeline::default()
    };
    let common = pipeline2.process().await;
    assert_eq!(
        common.err().unwrap().to_string(),
        "NotFound(\"`Command`s\")"
    )
}

#[tokio::test]
#[should_panic]
async fn one_empty_echo_task_one_command_pipeline_test() {
    let pipeline3: Pipeline = Pipeline {
        name: String::from(env!("CARGO_PKG_NAME")),
        version: String::from(env!("CARGO_PKG_VERSION")),
        description: String::from(env!("CARGO_PKG_DESCRIPTION")),
        url: String::from(env!("CARGO_PKG_REPOSITORY")),
        env: None,
        pipe: None,
        tasks: Some(indexmap::indexmap! { String::from("task0") => Task {
            commands: vec![CommandArgs::Echo(CommonContent {
                    content: None,
                    env: None,
                }),
            ],
            input_schema: None,
            output_schema: None,
            env: None,
        } }),
        schemas: None,
        ..Pipeline::default()
    };

    let _ = pipeline3.process().await.unwrap();
}

#[tokio::test]
async fn one_echo_task_pipeline_test() {
    let pipeline4: Pipeline = Pipeline {
        name: String::from(env!("CARGO_PKG_NAME")),
        version: String::from(env!("CARGO_PKG_VERSION")),
        description: String::from(env!("CARGO_PKG_DESCRIPTION")),
        url: String::from(env!("CARGO_PKG_REPOSITORY")),
        env: None,
        pipe: None,
        tasks: Some(indexmap::indexmap! {
        String::from("task0") =>
        Task {
            commands: vec![CommandArgs::Echo(CommonContent {
                    content: Some(serde_json::Value::String(String::from("FOO is set to ${FOO}"))),
                    env: Some(indexmap::indexmap! {
                        String::from("FOO") => serde_json::Value::String(String::from("bar"))
                    }),
                })],
            input_schema: None,
            output_schema: None,
            env: None,
        }
        }),
        schemas: None,
        ..Pipeline::default()
    };
    let common = pipeline4.process().await.unwrap();
    let env = common.env.unwrap();
    assert_eq!(
        env.get("PREVIOUS_TASK_NAME").unwrap(),
        &serde_json::Value::String(String::from("task0"))
    );
    assert_eq!(
        env.get("PREVIOUS_TASK_CONTENT").unwrap(),
        &serde_json::Value::String(String::from("FOO is set to bar"))
    )
}

#[tokio::test]
async fn one_http_task_pipeline_test() {
    let message_input = Message {
        message: String::from("greetings to ${ME}"),
    };
    let pipeline5: Pipeline = Pipeline {
        name: String::from(env!("CARGO_PKG_NAME")),
        version: String::from(env!("CARGO_PKG_VERSION")),
        description: String::from(env!("CARGO_PKG_DESCRIPTION")),
        url: String::from(env!("CARGO_PKG_REPOSITORY")),
        env: None,
        pipe: None,
        tasks: Some(indexmap::indexmap! {
        String::from("task0") =>
        Task {
            commands: vec![CommandArgs::HttpClient(HttpCommandArgs::new(
                    HttpArgs {
                        url: format!("{}/post", HTTPBIN_URL)
                            .parse::<http::uri::Uri>()
                            .unwrap(),
                        method: http::method::Method::POST,
                        headers: Some(vec![indexmap::indexmap! {
                            String::from("Content-Type") => serde_json_extensions::ValueNoObjOrArr::String(String::from("application/json")),
                        }]),
                    },
                    CommonContent {
                        content: Some(serde_json::to_value(&message_input).unwrap()),
                        env: None,
                    },
                    Default::default(),
                ))],
            input_schema: None,
            output_schema: None,
            env: None,
        }
        }),
        schemas: None,
        ..Pipeline::default()
    };
    let common = pipeline5.process().await.unwrap();
    let env = common.env.unwrap();
    assert_eq!(
        env.get("PREVIOUS_TASK_TYPE")
            .unwrap()
            .to_owned()
            .as_str()
            .unwrap(),
        "JSON"
    );
    assert_eq!(
        env.get("PREVIOUS_TASK_NAME").unwrap().as_str().unwrap(),
        "task0"
    );
    let http_bin_post_response: HttpBinPostResponse =
        serde_json::from_value(env.get("PREVIOUS_TASK_CONTENT").unwrap().to_owned()).unwrap();
    let message_response: Message =
        serde_json::from_str(http_bin_post_response.data.as_str()).unwrap();
    assert_eq!(message_response, message_input);
}

#[tokio::test]
async fn one_echo_one_http_command_in_one_task_pipeline_test() {
    let message_input = Message {
        message: String::from("greetings to ${ME}"),
    };
    let pipeline5: Pipeline = Pipeline {
        name: String::from(env!("CARGO_PKG_NAME")),
        version: String::from(env!("CARGO_PKG_VERSION")),
        description: String::from(env!("CARGO_PKG_DESCRIPTION")),
        url: String::from(env!("CARGO_PKG_REPOSITORY")),
        env: None,
        pipe: None,
        tasks: Some(indexmap::indexmap! {
        String::from("task0") =>
        Task {
            commands: vec![
                CommandArgs::Echo(CommonContent {
                        content: Some(serde_json::to_value(&message_input).unwrap()),
                        env: Some(indexmap::indexmap! {
                            String::from("ME") => serde_json::Value::String(String::from("Omega"))
                        }),
                    }),
                CommandArgs::HttpClient(HttpCommandArgs::new(
                        HttpArgs {
                            url: format!("{}/post", HTTPBIN_URL)
                                .parse::<http::uri::Uri>()
                                .unwrap(),
                            method: http::method::Method::POST,
                            headers: Some(vec![indexmap::indexmap! {
                                String::from("Content-Type") => serde_json_extensions::ValueNoObjOrArr::String(String::from("application/json")),
                            }]),
                        },
                        CommonContent {
                            content: None,
                            /* defaults to "-", i.e., the result of the previous task */
                            env: None,
                        },
                        Default::default(),
                    )),
            ],
            input_schema: None,
            output_schema: None,
            env: None,
        }
        }),
        schemas: None,
        ..Pipeline::default()
    };
    /*
    println!("{}", serde_json::to_string_pretty(&pipeline5).unwrap());
    println!("{}", toml::to_string(&pipeline5).unwrap());
    */
    let common = pipeline5.process().await.unwrap();
    let env = common.env.unwrap();
    assert_eq!(
        env.get("PREVIOUS_TASK_TYPE").unwrap().as_str().unwrap(),
        String::from("JSON")
    );
    assert_eq!(
        env.get("PREVIOUS_TASK_NAME").unwrap().as_str().unwrap(),
        String::from("task0")
    );
    let previous_task_content = env.get("PREVIOUS_TASK_CONTENT").unwrap();
    let httpbin_post_response: crate::test_models::HttpBinPostResponse<Message> =
        serde_json::from_value(previous_task_content.to_owned()).unwrap();
    assert_eq!(
        httpbin_post_response.json,
        Message {
            message: String::from("greetings to Omega"),
        }
    );
}

#[tokio::test]
async fn one_set_env_one_echo_one_http_command_one_jaq_in_one_task_pipeline_test() {
    let message_input = Message {
        message: String::from("greetings to ${ME}"),
    };
    let pipeline6: Pipeline = Pipeline {
        name: String::from(env!("CARGO_PKG_NAME")),
        version: String::from(env!("CARGO_PKG_VERSION")),
        description: String::from(env!("CARGO_PKG_DESCRIPTION")),
        url: String::from(env!("CARGO_PKG_REPOSITORY")),
        env: None,
        pipe: None,
        tasks: Some(indexmap::indexmap! {
        String::from("task0") => Task {
            commands: vec![
                CommandArgs::SetEnv(CommonContent {
                        content: None,
                        env: Some(indexmap::indexmap! {
                            String::from("ME") => serde_json::Value::String(String::from("Omega"))
                        })
                    }),
                CommandArgs::Echo(CommonContent {
                        content: Some(serde_json::to_value(&message_input).unwrap()),
                        env: None,
                    }),
                CommandArgs::HttpClient(HttpCommandArgs::new(
                        HttpArgs {
                            url: format!("{}/post", HTTPBIN_URL)
                                .parse::<http::uri::Uri>()
                                .unwrap(),
                            method: http::method::Method::POST,
                            headers: Some(vec![indexmap::indexmap! {
                                String::from("Content-Type") => serde_json_extensions::ValueNoObjOrArr::String(String::from("application/json")),
                            }]),
                        },
                        CommonContent {
                            content: None,
                            /* defaults to "-", i.e., the result of the previous task */
                            env: None,
                        },
                        Default::default(),
                    )),
                    CommandArgs::Jaq(CommonContent {
                        content: Some(serde_json::Value::String(String::from(".json.message"))),
                        env: None,
                    })
            ],
            input_schema: None,
            output_schema: None,
            env: None,
        }
        }),
        schemas: None,
        ..Pipeline::default()
    };
    let common = pipeline6.process().await.unwrap();
    let env = common.env.unwrap();
    /*
    println!("{}", serde_json::to_string(&pipeline6).unwrap());
    println!("{}", toml::to_string(&pipeline6).unwrap());
    */
    assert_eq!(
        env.get("PREVIOUS_TASK_NAME")
            .unwrap()
            .as_str()
            .unwrap()
            .to_owned(),
        String::from("task0")
    );
    let jaqed_message_response: String = env
        .get("PREVIOUS_TASK_CONTENT")
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();
    assert_eq!(
        jaqed_message_response,
        String::from("\"greetings to Omega\"")
    );
}
