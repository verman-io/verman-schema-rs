use std::str::FromStr;

use crate::commands::shared::make_subst_map;
use crate::errors::VermanSchemaError;
use crate::models::{CommonContent, HttpCommandArgs};

/// http command.
/// Note that because `CommonContent.content` is `serde_json::Value`, for non JSON inputs or outputs,
/// you'll want to create a new `http` command, possibly that accepts Vec<u8>.
/// (which can be done by removing `.json(` below and using a `serde_json::Value` with
/// `serde_json::Value::Array(Vec::<serde_json::Value::Number()>::new())`)
pub async fn http(
    http_command_args: &HttpCommandArgs,
) -> Result<(Option<reqwest::Response>, CommonContent), VermanSchemaError> {
    /*******************
     * Prepare request *
     *******************/
    let mut body = http_command_args.common_content.content.to_owned();
    let env = http_command_args
        .common_content
        .env
        .to_owned()
        .unwrap_or_else(|| indexmap::IndexMap::<String, serde_json::Value>::new());
    body = body.or_else(|| env.get("PREVIOUS_TASK_CONTENT").cloned());

    let mut args = http_command_args.args.to_owned();
    if !env.is_empty() {
        /* Do interpolation and ensure input is set */
        let variables = make_subst_map(&env);

        args.method = http::method::Method::from_str(
            subst::substitute(args.method.to_string().as_str(), &variables)?.as_str(),
        )?;
        args.url = http::uri::Uri::from_str(
            subst::substitute(args.url.to_string().as_str(), &variables)?.as_str(),
        )?;

        match body {
            Some(serde_json::Value::String(bod)) => {
                body = Some(serde_json::Value::String(subst::substitute(
                    bod.as_str(),
                    &variables,
                )?));
            }
            Some(val) => {
                body = Some(serde_json::from_str(
                    subst::substitute(serde_json::to_string(&val)?.as_str(), &variables)?.as_str(),
                )?);
            }
            _ => {}
        }
    }
    let client = reqwest::Client::new();
    let mut req = client.request(args.method, args.url.to_string());
    if let Some(headers) = args.headers {
        req = req.headers(indexmap_of_ValueNoObj_to_HeaderMap(&headers)?);
    }
    if let Some(bod) = body {
        req = req.json(&bod);
    }
    /*******************************
     * Execute then check response *
     *******************************/
    // log::info!("{:#?}", req);
    let res = req.send().await?;

    let status_code = res.status().as_u16();
    if status_code != http_command_args.expectation.status_code {
        return Err(VermanSchemaError::HttpError(status_code));
    }
    let headers = res.headers().clone();
    let content_type_opt = headers.get("Content-Type").map(|e| e.to_str().unwrap());
    match content_type_opt {
        Some("application/json") | Some("text/json") => Ok((
            None,
            CommonContent {
                env: Some(indexmap::IndexMap::<String, serde_json::Value>::from([
                    (
                        String::from("PREVIOUS_TASK_CONTENT"),
                        serde_json::from_str::<serde_json::Value>(
                            res./*json()*/text().await?.as_str(),
                        )?,
                    ),
                    (
                        String::from("PREVIOUS_TASK_TYPE"),
                        serde_json::Value::String(String::from("JSON")),
                    ),
                ])),
                ..CommonContent::default()
            },
        )),
        content_type @ (Some("text/plain")
        | Some("text/css")
        | Some("text/csv")
        | Some("text/html")
        | Some("text/javascript")
        | Some("text/xml")
        | Some("application/xml")) => {
            let task_type = content_type.unwrap();
            Ok((
                None,
                CommonContent {
                    env: Some(indexmap::indexmap! {
                        String::from("PREVIOUS_TASK_CONTENT") => serde_json::Value::String(res.text().await?),
                        String::from("PREVIOUS_TASK_TYPE") => serde_json::Value::String(task_type.to_string())
                    }),
                    ..CommonContent::default()
                },
            ))
        }
        content_type @ _ => Ok((
            None,
            CommonContent {
                env: Some(indexmap::indexmap! {
                    String::from("PREVIOUS_TASK_CONTENT") => serde_json::from_slice(res.bytes().await?.iter().as_slice())?,
                    String::from("PREVIOUS_TASK_TYPE") => serde_json::Value::String(String::from(content_type.unwrap()))
                }),
                ..CommonContent::default()
            },
        )),
    }
}

#[allow(non_snake_case)]
fn indexmap_of_ValueNoObj_to_HeaderMap(
    v: &Vec<indexmap::IndexMap<String, serde_json_extensions::ValueNoObjOrArr>>,
) -> Result<http::header::HeaderMap, VermanSchemaError> {
    let mut headers = http::header::HeaderMap::with_capacity(v.len());
    for index_map in v.iter() {
        for (k, v) in index_map.iter() {
            let header_value = header_value_from_value_no_obj(v)?;
            headers.insert(
                http::header::HeaderName::from_str(k.as_str())?,
                header_value,
            );
        }
    }
    Ok(headers)
}

fn header_value_from_value_no_obj(
    v: &serde_json_extensions::ValueNoObjOrArr,
) -> Result<http::header::HeaderValue, VermanSchemaError> {
    match v {
        serde_json_extensions::ValueNoObjOrArr::Null => {
            Ok(http::header::HeaderValue::from_bytes(b"")?)
        }
        serde_json_extensions::ValueNoObjOrArr::Bool(b) => {
            Ok(http::header::HeaderValue::from_bytes(if *b {
                b"1"
            } else {
                b"0"
            })?)
        }
        serde_json_extensions::ValueNoObjOrArr::Number(n) => {
            let serde_json_extensions::number::Number { n: n_inner } = n;
            Ok(if cfg!(not(feature = "arbitrary_precision")) {
                match n_inner {
                    serde_json_extensions::number::N::PosInt(p) => {
                        http::header::HeaderValue::from(*p)
                    }
                    serde_json_extensions::number::N::NegInt(n) => {
                        http::header::HeaderValue::from(*n)
                    }
                    serde_json_extensions::number::N::Float(_f) => unimplemented!(),
                    // http::header::HeaderValue::from_bytes(&*f.to_ne_bytes()),
                }
            } else {
                unimplemented!("arbitrary_precision `String`")
            })
        }
        serde_json_extensions::ValueNoObjOrArr::String(s) => {
            Ok(http::header::HeaderValue::try_from(s)?)
        } // ValueNoObjOrArr::Array(vec) => Ok(HeaderValue::try_from(vec.into())?)
    }
}

#[cfg(test)]
#[path = "http_client_test.rs"]
mod tests;
