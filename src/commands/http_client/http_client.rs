use std::str::FromStr;

use crate::errors::VermanSchemaError;
use crate::models::{CommonContent, HttpCommandArgs};

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
        .unwrap_or_else(|| indexmap::IndexMap::<String, either::Either<String, Vec<u8>>>::new());
    body = body.or(env.get("PREVIOUS_TASK_CONTENT").map(|s_or_v| match s_or_v {
        either::Either::Left(s) => s.to_owned().into_bytes(),
        either::Either::Right(v) => v.to_owned(),
    }));

    let mut args = http_command_args.args.to_owned();
    if !env.is_empty() {
        /* Do interpolation and ensure input is set */
        let mut hm = std::collections::HashMap::<String, String>::with_capacity(env.len());
        for (k, either_v) in env.iter() {
            if let either::Either::Left(v) = either_v {
                hm.insert(k.to_owned(), v.to_string());
            }
        }

        args.method = http::method::Method::from_str(
            subst::substitute(args.method.to_string().as_str(), &hm)?.as_str(),
        )?;
        args.url = http::uri::Uri::from_str(
            subst::substitute(args.url.to_string().as_str(), &hm)?.as_str(),
        )?;

        if let Some(bod) = body {
            body = Some(
                subst::substitute(std::str::from_utf8(bod.as_slice())?, &hm)?
                    .as_bytes()
                    .to_vec(),
            );
        }
    }
    let client = reqwest::Client::new();
    let mut req = client.request(args.method, args.url.to_string());
    if let Some(headers) = args.headers {
        req = req.headers(indexmap_of_ValueNoObj_to_HeaderMap(&headers)?);
    }
    if let Some(bod) = body {
        req = req.body(bod);
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
                env: Some(indexmap::indexmap! {
                    String::from("PREVIOUS_TASK_CONTENT") => either::Left(res./*json()*/text().await?),
                    String::from("PREVIOUS_TASK_TYPE") => either::Left(String::from("JSON"))
                }),
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
                        String::from("PREVIOUS_TASK_CONTENT") => either::Left(res.text().await?),
                        String::from("PREVIOUS_TASK_TYPE") => either::Left(String::from(task_type))
                    }),
                    ..CommonContent::default()
                },
            ))
        }
        content_type @ _ => Ok((
            None,
            CommonContent {
                env: Some(indexmap::indexmap! {
                    String::from("PREVIOUS_TASK_CONTENT") => either::Right(res.bytes().await?.to_vec()),
                    String::from("PREVIOUS_TASK_TYPE") => either::Left(String::from(content_type.unwrap()))
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
