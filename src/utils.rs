pub(crate) fn utf8_or_vecu8_of_value(v: &[u8]) -> serde_json::Value {
    std::str::from_utf8(v)
        .map(|s| serde_json::Value::String(s.to_owned()))
        .unwrap_or_else(|_| {
            serde_json::Value::Array(Vec::<serde_json::value::Value>::from_iter(v.into_iter().map(|num| -> serde_json::Number { num.to_owned().into() }).map(|e| -> serde_json::Value { serde_json::Value::Number(e) })))
        })
}
