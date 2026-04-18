use anyhow::Result;
use serde::Serialize;

pub fn render<T: Serialize>(data: &T, json: bool) -> Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(data)?);
    } else {
        let value = serde_json::to_value(data)?;
        match value {
            serde_json::Value::Object(map) => {
                for (key, value) in map {
                    println!("{key}: {}", format_value(&value));
                }
            }
            other => println!("{}", format_value(&other)),
        }
    }
    Ok(())
}

fn format_value(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Array(items) => items
            .iter()
            .map(format_value)
            .collect::<Vec<_>>()
            .join(", "),
        serde_json::Value::String(value) => value.clone(),
        _ => value.to_string(),
    }
}
