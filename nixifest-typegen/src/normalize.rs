use std::collections::HashSet;

use anyhow::{Result, bail};
use serde_json::{Map, Value};

pub fn normalize(schema: &Value, definitions: &Map<String, Value>) -> Result<Value> {
    let mut stack = HashSet::new();
    normalize_inner(schema, definitions, &mut stack)
}

fn normalize_inner(
    schema: &Value,
    definitions: &Map<String, Value>,
    stack: &mut HashSet<String>,
) -> Result<Value> {
    let Some(object) = schema.as_object() else {
        return Ok(schema.clone());
    };
    validate_all_of(object)?;

    if let Some(reference) = object.get("$ref").and_then(Value::as_str) {
        let name = reference
            .strip_prefix("#/components/schemas/")
            .or_else(|| reference.strip_prefix("#/definitions/"))
            .unwrap_or(reference);
        if !stack.insert(name.to_owned()) {
            return Ok(serde_json::json!({
                "type": "object",
                "x-nixifest-recursive": true,
            }));
        }
        let target = definitions
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("unresolved schema reference: {reference}"))?;
        let mut resolved = normalize_inner(target, definitions, stack)?;
        stack.remove(name);
        if object.len() > 1 {
            let siblings = normalize_inner(&without(object, &["$ref"]), definitions, stack)?;
            resolved = merge_values(resolved, siblings)?;
        }
        return Ok(resolved);
    }

    let mut result = Map::new();
    if let Some(all_of) = object.get("allOf").and_then(Value::as_array) {
        let mut merged = Value::Object(Map::new());
        for item in all_of {
            let normalized = normalize_inner(item, definitions, stack)?;
            merged = merge_values(merged, normalized)?;
        }
        if let Value::Object(all_of_result) = merged {
            result = all_of_result;
        }
    }
    let mut siblings = Map::new();
    for (key, value) in object {
        if key != "$ref" && key != "allOf" {
            let value = match key.as_str() {
                "properties" | "oneOf" | "anyOf" => value.clone(),
                "items" => normalize_inner(value, definitions, stack)?,
                "additionalProperties" if value.is_object() => {
                    normalize_inner(value, definitions, stack)?
                }
                _ => value.clone(),
            };
            siblings.insert(key.clone(), value);
        }
    }
    result = match merge_values(Value::Object(result), Value::Object(siblings))? {
        Value::Object(result) => result,
        _ => unreachable!("schema merges must produce objects"),
    };
    if let Some(properties) = result.get_mut("properties").and_then(Value::as_object_mut) {
        for value in properties.values_mut() {
            *value = normalize_inner(value, definitions, stack)?;
        }
    }
    if let Some(items) = result.get_mut("items") {
        *items = normalize_inner(items, definitions, stack)?;
    }
    if let Some(additional) = result.get_mut("additionalProperties")
        && additional.is_object()
    {
        *additional = normalize_inner(additional, definitions, stack)?;
    }
    for key in ["oneOf", "anyOf"] {
        if let Some(items) = result.get_mut(key).and_then(Value::as_array_mut) {
            for item in items {
                *item = normalize_inner(item, definitions, stack)?;
            }
        }
    }
    Ok(Value::Object(result))
}

fn without(object: &Map<String, Value>, keys: &[&str]) -> Value {
    Value::Object(
        object
            .iter()
            .filter(|(key, _)| !keys.contains(&key.as_str()))
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect(),
    )
}

fn validate_all_of(object: &Map<String, Value>) -> Result<()> {
    let Some(value) = object.get("allOf") else {
        return Ok(());
    };
    let Some(items) = value.as_array() else {
        bail!("OpenAPI `allOf` must be an array");
    };
    if items.is_empty() {
        bail!("OpenAPI `allOf` must not be empty");
    }
    if items.iter().any(|item| !item.is_object()) {
        bail!("OpenAPI `allOf` entries must be objects");
    }
    Ok(())
}

fn merge_values(left: Value, right: Value) -> Result<Value> {
    match (left, right) {
        (Value::Object(mut left), Value::Object(right)) => {
            for (key, value) in right {
                match (left.remove(&key), value) {
                    (Some(Value::Object(left_properties)), Value::Object(right_properties))
                        if key == "properties" =>
                    {
                        let merged = merge_values(
                            Value::Object(left_properties),
                            Value::Object(right_properties),
                        )?;
                        left.insert(key, merged);
                    }
                    (Some(Value::Array(left_required)), Value::Array(right_required))
                        if key == "required" =>
                    {
                        let mut required = left_required;
                        for item in right_required {
                            if !required.contains(&item) {
                                required.push(item);
                            }
                        }
                        left.insert(key, Value::Array(required));
                    }
                    (Some(existing), value) if existing == value => {
                        left.insert(key, value);
                    }
                    (Some(_), value)
                        if matches!(key.as_str(), "description" | "x-kubernetes-map-type") =>
                    {
                        left.insert(key, value);
                    }
                    (Some(_), _) => {
                        bail!("conflicting allOf schemas for `{key}`");
                    }
                    (None, value) => {
                        left.insert(key, value);
                    }
                }
            }
            Ok(Value::Object(left))
        }
        (_, right) => Ok(right),
    }
}
