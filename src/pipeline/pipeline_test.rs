use crate::commands::{CommandArgs, CommandName};
use crate::models::{Command, CommonContent, HttpArgs, HttpCommandArgs, Pipeline, Task};
use crate::test_models::{HttpBinPostResponse, Message, HTTPBIN_URL};

lazy_static::lazy_static! {
    static ref PIPELINE1: Pipeline = Pipeline {
        name: String::from(env!("CARGO_PKG_NAME")),
        version: String::from(env!("CARGO_PKG_VERSION")),
        description: String::from(env!("CARGO_PKG_DESCRIPTION")),
        url: String::from(env!("CARGO_PKG_REPOSITORY")),
        env: None,
        pipe: vec![],
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
        pipe: vec![],
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
        pipe: vec![],
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
        pipe: vec![],
        tasks: Some(indexmap::indexmap! { String::from("task0") => Task {
            commands: vec![Command {
                cmd: CommandName::Echo,
                args: Some(CommandArgs::Echo(CommonContent {
                    content: None,
                    env: None,
                })),
            }],
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
        pipe: vec![],
        tasks: Some(indexmap::indexmap! {
        String::from("task0") =>
        Task {
            commands: vec![Command {
                cmd: CommandName::Echo,
                args: Some(CommandArgs::Echo(CommonContent {
                    content: Some(Vec::from(b"FOO is set to ${FOO}")),
                    env: Some(indexmap::indexmap! {
                        String::from("FOO") => either::Left(String::from("bar"))
                    }),
                })),
            }],
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
        &either::<String, Vec<u8>>::Left(String::from("task0"))
    );
    assert_eq!(
        env.get("PREVIOUS_TASK_CONTENT").unwrap(),
        &either::<String, Vec<u8>>::Left(String::from("FOO is set to bar"))
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
        pipe: vec![],
        tasks: Some(indexmap::indexmap! {
        String::from("task0") =>
        Task {
            commands: vec![Command {
                cmd: CommandName::HttpClient,
                args: Some(CommandArgs::HttpClient(HttpCommandArgs {
                    args: HttpArgs {
                        url: format!("{}/post", HTTPBIN_URL)
                            .parse::<http::uri::Uri>()
                            .unwrap(),
                        method: http::method::Method::POST,
                        headers: Some(vec![indexmap::indexmap! {
                            String::from("Content-Type") => serde_json_extensions::ValueNoObjOrArr::String(String::from("application/json")),
                        }]),
                    },
                    common_content: CommonContent {
                        content: Some(
                            serde_json::to_string(&message_input).unwrap().into_bytes(),
                        ),
                        env: None,
                    },
                    expectation: Default::default(),
                })),
            }],
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
            .left()
            .unwrap()
            .as_str(),
        "JSON"
    );
    assert_eq!(
        env.get("PREVIOUS_TASK_NAME")
            .unwrap()
            .to_owned()
            .left()
            .unwrap()
            .as_str(),
        "task0"
    );
    let http_bin_post_response: HttpBinPostResponse = serde_json::from_str(
        env.get("PREVIOUS_TASK_CONTENT")
            .unwrap()
            .to_owned()
            .left()
            .unwrap()
            .as_str(),
    )
    .unwrap();
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
        pipe: vec![],
        tasks: Some(indexmap::indexmap! {
        String::from("task0") =>
        Task {
            commands: vec![
                Command {
                    cmd: CommandName::Echo,
                    args: Some(CommandArgs::Echo(CommonContent {
                        content: Some(serde_json::to_vec(&message_input).unwrap()),
                        env: Some(indexmap::indexmap! {
                            String::from("ME") => either::Left(String::from("Omega"))
                        }),
                    })),
                },
                Command {
                    cmd: CommandName::HttpClient,
                    args: Some(CommandArgs::HttpClient(HttpCommandArgs {
                        args: HttpArgs {
                            url: format!("{}/post", HTTPBIN_URL)
                                .parse::<http::uri::Uri>()
                                .unwrap(),
                            method: http::method::Method::POST,
                            headers: Some(vec![indexmap::indexmap! {
                                String::from("Content-Type") => serde_json_extensions::ValueNoObjOrArr::String(String::from("application/json")),
                            }]),
                        },
                        common_content: CommonContent {
                            content: None,
                            /* defaults to "-", i.e., the result of the previous task */
                            env: None,
                        },
                        expectation: Default::default(),
                    })),
                },
            ],
            input_schema: None,
            output_schema: None,
            env: None,
        }
        }),
        schemas: None,
        ..Pipeline::default()
    };
    /* println!("{}", serde_json::to_string(&pipeline5).unwrap());
    println!("{}", toml::to_string(&pipeline5).unwrap()); */
    let common = pipeline5.process().await.unwrap();
    let env = common.env.unwrap();
    assert_eq!(
        env.get("PREVIOUS_TASK_TYPE")
            .unwrap()
            .to_owned()
            .left()
            .unwrap(),
        String::from("JSON")
    );
    assert_eq!(
        env.get("PREVIOUS_TASK_NAME")
            .unwrap()
            .to_owned()
            .left()
            .unwrap(),
        String::from("task0")
    );
    let http_bin_post_response: HttpBinPostResponse = serde_json::from_str(
        env.get("PREVIOUS_TASK_CONTENT")
            .unwrap()
            .to_owned()
            .left()
            .unwrap()
            .as_str(),
    )
    .unwrap();
    let message_response: Message =
        serde_json::from_str(http_bin_post_response.data.as_str()).unwrap();
    assert_eq!(
        message_response,
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
        pipe: vec![],
        tasks: Some(indexmap::indexmap! {
        String::from("task0") => Task {
            commands: vec![
                Command {
                    cmd: CommandName::SetEnv,
                    args: Some(CommandArgs::SetEnv(CommonContent {
                        content: None,
                        env: Some(indexmap::indexmap! {
                            String::from("ME") => either::Left(String::from("Omega"))
                        })
                    }))
                },
                Command {
                    cmd: CommandName::Echo,
                    args: Some(CommandArgs::Echo(CommonContent {
                        content: Some(serde_json::to_vec(&message_input).unwrap()),
                        env: None,
                    })),
                },
                Command {
                    cmd: CommandName::HttpClient,
                    args: Some(CommandArgs::HttpClient(HttpCommandArgs {
                        args: HttpArgs {
                            url: format!("{}/post", HTTPBIN_URL)
                                .parse::<http::uri::Uri>()
                                .unwrap(),
                            method: http::method::Method::POST,
                            headers: Some(vec![indexmap::indexmap! {
                                String::from("Content-Type") => serde_json_extensions::ValueNoObjOrArr::String(String::from("application/json")),
                            }]),
                        },
                        common_content: CommonContent {
                            content: None,
                            /* defaults to "-", i.e., the result of the previous task */
                            env: None,
                        },
                        expectation: Default::default(),
                    })),
                },
                    Command {
                    cmd: CommandName::Jaq,
                    args: Some(CommandArgs::Jaq(CommonContent {
                        content: Some(b".json.message".to_vec()),
                        env: None,
                    })),
                },
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
    println!("{}", serde_json::to_string(&pipeline6).unwrap());
    println!("{}", toml::to_string(&pipeline6).unwrap());
    assert_eq!(
        env.get("PREVIOUS_TASK_NAME")
            .unwrap()
            .to_owned()
            .left()
            .unwrap(),
        String::from("task0")
    );
    let jaqed_message_response: String = env
        .get("PREVIOUS_TASK_CONTENT")
        .unwrap()
        .to_owned()
        .left()
        .unwrap();
    assert_eq!(
        jaqed_message_response,
        String::from("\"greetings to Omega\"")
    );
}
