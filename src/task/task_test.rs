use crate::commands::CommandArgs;
use crate::models::{CommonContent, Task};

#[test]
fn test_task_string_env_value() {
    let task0 = Task {
        commands: vec![CommandArgs::Echo(CommonContent {
            content: Some(serde_json::Value::String(String::from("${math_messages}"))),
            env: Some(indexmap::indexmap! {
                String::from("Foo") => serde_json::Value::String(String::from("Bar"))
            }),
        })],
        ..Task::default()
    };
    assert_eq!(
        serde_json::to_string(&task0).unwrap(),
        "{\"commands\":[{\"cmd\":\"Echo\",\"content\":\"${math_messages}\",\"env\":{\"Foo\":\"Bar\"}}]}"
    );
}

const TASK1_S: &'static str = "{\"commands\":[{\"cmd\":\"Echo\",\"content\":\"${math_messages}\",\"env\":{\"weird\":\"[{\\\"cmd\\\":\\\"Echo\\\",\\\"content\\\":\\\"${math_messages}\\\",\\\"env\\\":{\\\"Foo\\\":\\\"Bar\\\"}}]\"}}]}";
const TASK1_S_RAW: &'static str = r###"{"commands":[{"cmd":"Echo","content":"${math_messages}","env":{"weird":"[{\"cmd\":\"Echo\",\"content\":\"${math_messages}\",\"env\":{\"Foo\":\"Bar\"}}]"}}]}"###;

#[test]
fn test_task_json_env_value() {
    let task1 = Task {
        commands: vec![CommandArgs::Echo(CommonContent {
            content: Some(serde_json::Value::String(String::from("${math_messages}"))),
            env: Some(indexmap::indexmap! {
                String::from("weird") => serde_json::Value::String(String::from("[{\"cmd\":\"Echo\",\"content\":\"${math_messages}\",\"env\":{\"Foo\":\"Bar\"}}]"))
            }),
        })],
        ..Task::default()
    };
    assert_eq!(serde_json::to_string(&task1).unwrap(), TASK1_S);
    assert_eq!(TASK1_S_RAW, TASK1_S);
    assert_eq!(serde_json::from_str::<Task>(TASK1_S).unwrap(), task1);
}

#[test]
fn test_task_serialise_into_env_from_json_value() {
    let task2_s = "{\"commands\":[{\"cmd\":\"Echo\",\"content\":\"${math_messages}\",\"env\":{\"weird\":{\"word\": [5,6,7]}}}]}";
    println!("{}", task2_s);
    let task2: Task = serde_json::from_str(task2_s).unwrap();
    println!("{:?}", task2);
    let _task2_wanted = Task {
        commands: vec![CommandArgs::Echo(CommonContent {
            content: Some(serde_json::Value::String(String::from("${math_messages}"))),
            env: Some(indexmap::indexmap! {
                String::from("weird") => serde_json::Value::String(String::new())
            }),
        })],
        ..Task::default()
    };
}

/*
let cmd0: CommandArgs = serde_json::json!({
    "cmd": "Echo",
    "content": "${math_messages}",
    "env": {
    "math_messages": {
        "messages": [
        {}]}}})

 */
