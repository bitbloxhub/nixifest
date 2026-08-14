use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use serde_json::Value;

use crate::model::ResourceSchema;

pub fn load(inputs: &[PathBuf]) -> Result<Vec<ResourceSchema>> {
    let mut resources = BTreeMap::new();
    for input in inputs {
        let text = std::fs::read_to_string(input)
            .with_context(|| format!("reading {}", input.display()))?;
        for document in serde_yaml::Deserializer::from_str(&text) {
            let yaml = serde_yaml::Value::deserialize(document)
                .with_context(|| format!("parsing {}", input.display()))?;
            let value = serde_json::to_value(yaml)
                .with_context(|| format!("converting {}", input.display()))?;
            load_document(&value, &mut resources, input)?;
        }
    }
    if resources.is_empty() {
        bail!("CRD input contains no CustomResourceDefinition documents");
    }
    Ok(resources.into_values().collect())
}

fn load_document(
    document: &Value,
    resources: &mut BTreeMap<String, ResourceSchema>,
    source: &Path,
) -> Result<()> {
    let kind = document.get("kind").and_then(Value::as_str);
    if matches!(kind, Some("List" | "CustomResourceDefinitionList")) {
        let items = document
            .get("items")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                anyhow::anyhow!("{} document is missing array items", kind.unwrap_or("List"))
            })?;
        for item in items {
            load_document(item, resources, source)?;
        }
        return Ok(());
    }
    if document.get("kind").and_then(Value::as_str) != Some("CustomResourceDefinition") {
        return Ok(());
    }

    let crd_name = document
        .pointer("/metadata/name")
        .and_then(Value::as_str)
        .filter(|name| !name.is_empty())
        .unwrap_or("<unnamed>");
    let context = format!("CRD {crd_name} in {}", source.display());
    match document.get("apiVersion").and_then(Value::as_str) {
        Some("apiextensions.k8s.io/v1") => {}
        Some(api_version) => bail!("{context} has unsupported apiVersion: {api_version}"),
        None => bail!("{context} is missing apiVersion"),
    }
    if document.pointer("/spec/preserveUnknownFields") == Some(&Value::Bool(true)) {
        bail!("{context} uses unsupported spec.preserveUnknownFields=true");
    }
    let group = document
        .pointer("/spec/group")
        .and_then(Value::as_str)
        .filter(|group| !group.is_empty())
        .ok_or_else(|| anyhow::anyhow!("{context} is missing non-empty spec.group"))?;
    let kind = document
        .pointer("/spec/names/kind")
        .and_then(Value::as_str)
        .filter(|kind| !kind.is_empty())
        .ok_or_else(|| anyhow::anyhow!("{context} is missing non-empty spec.names.kind"))?;
    let namespaced = match document.pointer("/spec/scope").and_then(Value::as_str) {
        Some("Namespaced") => true,
        Some("Cluster") => false,
        Some(scope) => bail!("{context} has unknown scope: {scope}"),
        None => bail!("{context} is missing spec.scope"),
    };
    let versions = document
        .pointer("/spec/versions")
        .and_then(Value::as_array)
        .filter(|versions| !versions.is_empty())
        .ok_or_else(|| anyhow::anyhow!("{context} is missing non-empty spec.versions"))?;
    let mut emitted_served_version = false;
    for version in versions {
        let version_name = version
            .get("name")
            .and_then(Value::as_str)
            .filter(|name| !name.is_empty())
            .ok_or_else(|| {
                anyhow::anyhow!("{context} is missing non-empty spec.versions[].name")
            })?;
        let served = version
            .get("served")
            .and_then(Value::as_bool)
            .ok_or_else(|| {
                anyhow::anyhow!("{context} version {version_name} is missing boolean served")
            })?;
        let schema = version
            .pointer("/schema/openAPIV3Schema")
            .filter(|schema| schema.is_object())
            .cloned()
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "{context} version {version_name} is missing an object OpenAPI schema"
                )
            })?;
        if !served {
            continue;
        }
        emitted_served_version = true;
        let api_version = format!("{group}/{version_name}");
        let resource = ResourceSchema {
            api_version: api_version.clone(),
            group: group.to_owned(),
            version: version_name.to_owned(),
            kind: kind.to_owned(),
            namespaced,
            schema,
            definitions: Default::default(),
        };
        if resources
            .insert(format!("{api_version}/{kind}"), resource)
            .is_some()
        {
            bail!("{context} defines duplicate resource: {api_version}/{kind}");
        }
    }
    if !emitted_served_version {
        bail!("{context} has no served versions");
    }
    Ok(())
}

use serde::Deserialize;
