use anyhow::{Result, anyhow, bail};

use std::collections::BTreeMap;

use serde_json::Value;

use crate::model::ResourceSchema;
use crate::normalize::normalize;

pub fn emit_module(resources: &[ResourceSchema]) -> Result<String> {
    let mut output = String::from("{ lib, ... }: { options.resources = {");
    for resource in resources {
        let _ = (&resource.group, &resource.version, resource.namespaced);
        let mut schema = normalize(&resource.schema, &resource.definitions)?;
        strip_identity_fields(&mut schema);
        output.push_str(&format!(
            " {}.{} = lib.mkOption {{ default = {{ }}; ",
            nix_string(&resource.api_version),
            nix_string(&resource.kind),
        ));
        if let Some(description) = description(&schema) {
            output.push_str(&format!("description = {}; ", nix_string(description)));
        }
        output.push_str(&format!(
            "type = lib.types.attrsOf ({}); }};",
            resource_type(&schema)?,
        ));
    }
    output.push_str(" }; }");
    Ok(output)
}

fn strip_identity_fields(schema: &mut Value) {
    if let Some(required) = schema.get_mut("required").and_then(Value::as_array_mut) {
        required
            .retain(|name| name.as_str() != Some("apiVersion") && name.as_str() != Some("kind"));
    }
    let Some(properties) = schema.get_mut("properties").and_then(Value::as_object_mut) else {
        return;
    };
    properties.remove("apiVersion");
    properties.remove("kind");
    if let Some(metadata) = properties.get_mut("metadata") {
        if let Some(metadata_properties) = metadata
            .get_mut("properties")
            .and_then(Value::as_object_mut)
        {
            metadata_properties.remove("name");
        }
        if let Some(required) = metadata.get_mut("required").and_then(Value::as_array_mut) {
            required.retain(|name| name.as_str() != Some("name"));
        }
    }
}

fn required_properties(schema: &Value) -> std::collections::HashSet<&str> {
    schema
        .get("required")
        .and_then(Value::as_array)
        .map(|required| required.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default()
}

fn append_option(
    output: &mut String,
    name: &str,
    schema_type: &str,
    description: Option<&str>,
    required: bool,
) {
    output.push_str(&format!(" {} = lib.mkOption {{ type = ", nix_string(name)));
    if required {
        output.push_str(&format!("{schema_type};"));
    } else {
        output.push_str(&format!(
            "lib.types.nullOr ({schema_type}); default = null;",
        ));
    }
    if let Some(description) = description {
        output.push_str(&format!(" description = {};", nix_string(description)));
    }
    output.push_str(" }");
}

fn description(schema: &Value) -> Option<&str> {
    schema
        .get("description")
        .and_then(Value::as_str)
        .filter(|description| !description.is_empty())
}

fn validated_schema_type(schema: &Value) -> Result<Option<&str>> {
    let Some(value) = schema.get("type") else {
        return Ok(None);
    };
    let schema_type = value
        .as_str()
        .ok_or_else(|| anyhow!("OpenAPI schema type must be a string"))?;
    if !matches!(
        schema_type,
        "string" | "integer" | "number" | "boolean" | "array" | "object"
    ) {
        bail!("unsupported OpenAPI schema type: {schema_type}");
    }
    Ok(Some(schema_type))
}

fn nix_type(schema: &Value) -> Result<String> {
    let schema_type = nix_non_nullable_type(schema)?;
    if schema.get("nullable") == Some(&Value::Bool(true)) {
        Ok(format!("lib.types.nullOr ({schema_type})"))
    } else {
        Ok(schema_type)
    }
}

fn validate_schema_shape(schema: &Value) -> Result<Option<&str>> {
    if !schema.is_object() {
        bail!("OpenAPI schema must be an object");
    }
    let schema_type = validated_schema_type(schema)?;
    if let Some(properties) = schema.get("properties") {
        let Some(properties) = properties.as_object() else {
            bail!("OpenAPI schema properties must be an object");
        };
        if properties.values().any(|property| !property.is_object()) {
            bail!("OpenAPI property schemas must be objects");
        }
    }
    if let Some(required) = schema.get("required") {
        let Some(required) = required.as_array() else {
            bail!("OpenAPI schema required must be an array");
        };
        if required.iter().any(|name| !name.is_string()) {
            bail!("OpenAPI schema required entries must be strings");
        }
    }
    if let Some(items) = schema.get("items")
        && !items.is_object()
    {
        bail!("OpenAPI array items schema must be an object");
    }
    if let Some(additional) = schema.get("additionalProperties")
        && !additional.is_boolean()
        && !additional.is_object()
    {
        bail!("OpenAPI additionalProperties must be a boolean or object");
    }
    if let Some(values) = schema.get("enum") {
        let Some(values) = values.as_array() else {
            bail!("OpenAPI enum must be an array");
        };
        if values.is_empty() {
            bail!("OpenAPI enum must not be empty");
        }
    }
    for key in [
        "nullable",
        "x-kubernetes-embedded-resource",
        "x-kubernetes-int-or-string",
        "x-kubernetes-preserve-unknown-fields",
    ] {
        if let Some(value) = schema.get(key)
            && !value.is_boolean()
        {
            bail!("OpenAPI `{key}` must be a boolean");
        }
    }
    validate_schema_combinators(schema)?;
    Ok(schema_type)
}

fn json_number_is_integer(value: &serde_json::Number) -> bool {
    if value.as_i64().is_some() || value.as_u64().is_some() {
        return true;
    }
    let value = value.to_string();
    if value.contains(['e', 'E']) {
        return true;
    }
    let Some((_, fraction)) = value.split_once('.') else {
        return false;
    };
    fraction.chars().all(|character| character == '0')
}

fn enum_value_matches_type(value: &Value, schema_type: &str) -> bool {
    match schema_type {
        "string" => value.is_string(),
        "integer" => value.as_number().is_some_and(json_number_is_integer),
        "number" => value.is_number(),
        "boolean" => value.is_boolean(),
        "array" => value.is_array(),
        "object" => value.is_object(),
        _ => false,
    }
}

fn is_int_or_string_any_of(schema: &Value) -> bool {
    let Some(Value::Array(variants)) = schema.get("anyOf") else {
        return false;
    };
    if variants.len() != 2 {
        return false;
    }
    ["integer", "string"].iter().all(|schema_type| {
        variants.iter().any(|variant| {
            variant.as_object().is_some_and(|variant| {
                variant.len() == 1
                    && variant.get("type") == Some(&Value::String((*schema_type).to_owned()))
            })
        })
    })
}

fn nix_non_nullable_type(schema: &Value) -> Result<String> {
    let schema_type = validate_schema_shape(schema)?;
    let int_or_string = schema.get("x-kubernetes-int-or-string") == Some(&Value::Bool(true));
    if int_or_string {
        if schema_type.is_some()
            || schema.get("oneOf").is_some()
            || (schema.get("anyOf").is_some() && !is_int_or_string_any_of(schema))
        {
            bail!("x-kubernetes-int-or-string must use the supported union shape");
        }
        if schema.get("enum").is_some()
            || schema.get("x-kubernetes-preserve-unknown-fields") == Some(&Value::Bool(true))
            || schema.get("x-kubernetes-embedded-resource") == Some(&Value::Bool(true))
        {
            bail!("x-kubernetes-int-or-string has unsupported structural combinations");
        }
        return Ok("lib.types.oneOf [ lib.types.int lib.types.str ]".to_owned());
    }
    let union = schema.get("oneOf").or_else(|| schema.get("anyOf"));
    if let Some(union) = union {
        if schema_type.is_some()
            || schema.get("properties").is_some()
            || schema.get("required").is_some()
            || schema.get("additionalProperties").is_some()
            || schema.get("items").is_some()
            || schema.get("x-kubernetes-embedded-resource").is_some()
        {
            bail!("unions with outer structural constraints are not supported");
        }
        if schema.get("enum").is_some() {
            bail!("schemas combining enum with oneOf or anyOf are not supported");
        }
        if schema.get("x-kubernetes-preserve-unknown-fields") == Some(&Value::Bool(true)) {
            bail!(
                "schemas combining preserve-unknown-fields with oneOf or anyOf are not supported"
            );
        }
        let variants = union.as_array().expect("validated union must be an array");
        let variants = variants.iter().map(nix_type).collect::<Result<Vec<_>>>()?;
        return Ok(format!("lib.types.oneOf [ {} ]", variants.join(" ")));
    }
    if schema.get("x-kubernetes-preserve-unknown-fields") == Some(&Value::Bool(true)) {
        if schema_type.is_some_and(|schema_type| schema_type != "object") {
            bail!("x-kubernetes-preserve-unknown-fields requires an object schema");
        }
        if schema.get("enum").is_some() {
            bail!("schemas combining preserve-unknown-fields with enum are not supported");
        }
        if schema_type == Some("object")
            || schema.get("properties").is_some()
            || schema.get("additionalProperties").is_some()
        {
            return object_type(schema);
        }
        return Ok("lib.types.anything".to_owned());
    }
    if let Some(values) = schema.get("enum").and_then(Value::as_array) {
        if let Some(schema_type) = schema_type
            && values
                .iter()
                .any(|value| !enum_value_matches_type(value, schema_type))
        {
            bail!("enum values are incompatible with explicit OpenAPI type");
        }
        let values = values.iter().map(nix_value).collect::<Result<Vec<_>>>()?;
        return Ok(format!("lib.types.enum [ {} ]", values.join(" ")));
    }
    if schema.get("x-kubernetes-embedded-resource") == Some(&Value::Bool(true))
        && schema_type != Some("object")
    {
        bail!("x-kubernetes-embedded-resource requires type object");
    }
    // `not` is a validation constraint intentionally enforced by Kubernetes.
    match schema_type {
        Some("string") => Ok("lib.types.str".to_owned()),
        Some("integer") => Ok("lib.types.int".to_owned()),
        Some("number") => Ok("lib.types.number".to_owned()),
        Some("boolean") => Ok("lib.types.bool".to_owned()),
        Some("array") => {
            let item_type = schema
                .get("items")
                .map(nix_type)
                .transpose()?
                .unwrap_or_else(|| "lib.types.anything".to_owned());
            Ok(format!("lib.types.listOf ({item_type})"))
        }
        Some("object") => object_type(schema),
        None if schema.get("properties").is_some()
            || schema.get("additionalProperties").is_some() =>
        {
            object_type(schema)
        }
        Some(other) => bail!("unsupported OpenAPI schema type: {other}"),
        None => Ok("lib.types.anything".to_owned()),
    }
}

fn validate_schema_combinators(schema: &Value) -> Result<()> {
    if schema.get("oneOf").is_some() && schema.get("anyOf").is_some() {
        bail!("schemas combining oneOf and anyOf are not supported");
    }
    for key in ["oneOf", "anyOf"] {
        let Some(value) = schema.get(key) else {
            continue;
        };
        let Some(variants) = value.as_array() else {
            bail!("OpenAPI `{key}` must be an array");
        };
        if variants.is_empty() {
            bail!("OpenAPI `{key}` must not be empty");
        }
        if variants.iter().any(|variant| !variant.is_object()) {
            bail!("OpenAPI `{key}` entries must be objects");
        }
    }
    Ok(())
}

fn resource_type(schema: &Value) -> Result<String> {
    let schema_type = validate_schema_shape(schema)?;
    if schema.get("oneOf").is_some() {
        bail!("root resource schemas with oneOf are not supported");
    }
    if schema.get("anyOf").is_some() {
        bail!("root resource schemas with anyOf are not supported");
    }
    if let Some(schema_type) = schema_type {
        if schema_type != "object" {
            bail!("Kubernetes resource schema must have type object");
        }
    } else if schema
        .get("properties")
        .and_then(Value::as_object)
        .is_none()
        && schema.get("additionalProperties").is_none()
        && schema.get("x-kubernetes-preserve-unknown-fields") != Some(&Value::Bool(true))
    {
        bail!("Kubernetes resource schema must describe an object");
    }
    object_type(schema)
}

fn object_type(schema: &Value) -> Result<String> {
    let properties = schema.get("properties").and_then(Value::as_object);
    let embedded = schema.get("x-kubernetes-embedded-resource") == Some(&Value::Bool(true));
    if !embedded && !properties.is_some_and(|properties| !properties.is_empty()) {
        return object_map_type(schema);
    }
    let mut options = String::new();
    if embedded {
        for (name, property_type) in [
            ("apiVersion", "lib.types.str"),
            ("kind", "lib.types.str"),
            ("metadata", "lib.types.attrsOf lib.types.anything"),
        ] {
            if !properties.is_some_and(|properties| properties.contains_key(name)) {
                append_option(&mut options, name, property_type, None, false);
                options.push(';');
            }
        }
    }
    let required = required_properties(schema);
    if let Some(properties) = properties {
        for (name, property) in properties {
            let property_type = nix_type(property)?;
            append_option(
                &mut options,
                name,
                &property_type,
                description(property),
                required.contains(name.as_str()),
            );
            options.push(';');
        }
    }
    let freeform_type =
        if schema.get("x-kubernetes-preserve-unknown-fields") == Some(&Value::Bool(true)) {
            Some("lib.types.attrsOf lib.types.anything".to_owned())
        } else {
            match schema.get("additionalProperties") {
                Some(Value::Bool(true)) => Some("lib.types.attrsOf lib.types.anything".to_owned()),
                Some(Value::Object(_)) => {
                    let additional = schema.get("additionalProperties").unwrap();
                    Some(format!("lib.types.attrsOf ({})", nix_type(additional)?))
                }
                _ => None,
            }
        };
    let freeform_type = freeform_type
        .map(|schema_type| format!(" freeformType = {schema_type};"))
        .unwrap_or_default();
    Ok(format!(
        "lib.types.submodule {{ options = {{ {options} }};{freeform_type} }}",
    ))
}

fn object_map_type(schema: &Value) -> Result<String> {
    if schema.get("x-kubernetes-preserve-unknown-fields") == Some(&Value::Bool(true)) {
        return Ok("lib.types.attrsOf lib.types.anything".to_owned());
    }
    match schema.get("additionalProperties") {
        Some(Value::Bool(false)) => Ok("lib.types.submodule { options = { }; }".to_owned()),
        Some(Value::Bool(true)) | None => Ok("lib.types.attrsOf lib.types.anything".to_owned()),
        Some(Value::Object(_)) => {
            let additional = schema.get("additionalProperties").unwrap();
            Ok(format!("lib.types.attrsOf ({})", nix_type(additional)?))
        }
        _ => Ok("lib.types.attrsOf lib.types.anything".to_owned()),
    }
}

fn nix_string(value: &str) -> String {
    let mut output = String::with_capacity(value.len() + 2);
    output.push('"');
    for character in value.chars() {
        match character {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            '$' => output.push_str("\\$"),
            character => output.push(character),
        }
    }
    output.push('"');
    output
}

fn nix_number(value: &serde_json::Number) -> Result<String> {
    let value = value.to_string();
    if value.contains(['e', 'E']) {
        bail!("JSON number uses unsupported exponent notation: {value}");
    }
    if value.contains('.') {
        value
            .parse::<f64>()
            .ok()
            .filter(|value| value.is_finite())
            .ok_or_else(|| anyhow!("JSON number is not a finite Nix float: {value}"))?;
        return Ok(value);
    }
    if value.parse::<i64>().is_err() {
        bail!("JSON integer is outside Nix integer range: {value}");
    }
    Ok(value)
}

fn nix_value(value: &Value) -> Result<String> {
    match value {
        Value::Null => Ok("null".to_owned()),
        Value::Bool(value) => Ok(value.to_string()),
        Value::Number(value) => nix_number(value),
        Value::String(value) => Ok(nix_string(value)),
        Value::Array(values) => {
            let values = values.iter().map(nix_value).collect::<Result<Vec<_>>>()?;
            Ok(format!("[ {} ]", values.join(" ")))
        }
        Value::Object(values) => {
            let values: BTreeMap<_, _> = values.iter().collect();
            let values = values
                .iter()
                .map(|(key, value)| Ok(format!("{} = {};", nix_string(key), nix_value(value)?)))
                .collect::<Result<Vec<_>>>()?;
            Ok(format!("{{ {} }}", values.join(" ")))
        }
    }
}
