use std::collections::{BTreeMap, btree_map::Entry};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use serde_json::{Map, Value};

use crate::model::ResourceSchema;

pub fn load(inputs: &[PathBuf]) -> Result<Vec<ResourceSchema>> {
    let mut discovery = None;
    let mut documents = Vec::new();
    for input in inputs {
        if input.file_name().and_then(|name| name.to_str()) == Some("aggregated_v2.json") {
            if discovery.is_some() {
                bail!("multiple aggregated_v2.json files found");
            }
            discovery = Some(parse_discovery(input)?);
        } else if input.extension().and_then(|extension| extension.to_str()) == Some("json") {
            let document = read_json(input)?;
            if document.get("components").is_some() {
                documents.push(document);
            }
        }
    }
    if documents.is_empty() {
        bail!("Kubernetes input contains no OpenAPI v3 documents");
    }

    let mut resources = BTreeMap::new();
    for document in documents {
        let definitions = document
            .pointer("/components/schemas")
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default();
        for schema in definitions.values() {
            for gvk in group_version_kinds(schema) {
                let group = gvk.get("group").and_then(Value::as_str).unwrap_or("");
                let version = gvk.get("version").and_then(Value::as_str).unwrap_or("");
                let kind = gvk.get("kind").and_then(Value::as_str).unwrap_or("");
                if kind.is_empty() || version.is_empty() || kind.ends_with("List") {
                    continue;
                }
                let api_version = api_version(group, version);
                let key = format!("{api_version}/{kind}");
                let namespaced = if group.is_empty() {
                    let Some(namespaced) = collection_scope(&document, group, version, kind) else {
                        continue;
                    };
                    namespaced
                } else {
                    let namespaced = match &discovery {
                        Some(discovery) => discovery.get(&key).copied(),
                        None => collection_scope(&document, group, version, kind),
                    };
                    let Some(namespaced) = namespaced else {
                        continue;
                    };
                    namespaced
                };
                let resource = ResourceSchema {
                    api_version: api_version.clone(),
                    group: group.to_owned(),
                    version: version.to_owned(),
                    kind: kind.to_owned(),
                    namespaced,
                    schema: schema.clone(),
                    definitions: definitions.clone(),
                };
                match resources.entry(key.clone()) {
                    Entry::Vacant(entry) => {
                        entry.insert(resource);
                    }
                    Entry::Occupied(_) => {
                        bail!("multiple OpenAPI schemas found for resource {key}");
                    }
                }
            }
        }
    }
    if resources.is_empty() {
        bail!("Kubernetes input contains no resources");
    }
    Ok(resources.into_values().collect())
}

fn parse_discovery(path: &Path) -> Result<BTreeMap<String, bool>> {
    let document = read_json(path)?;
    let mut resources = BTreeMap::new();
    let items = document
        .get("items")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow::anyhow!("aggregated_v2.json is missing an `items` array"))?;
    for item in items {
        let fallback_group = item
            .pointer("/metadata/name")
            .and_then(Value::as_str)
            .unwrap_or("");
        let versions = item
            .get("versions")
            .and_then(Value::as_array)
            .ok_or_else(|| anyhow::anyhow!("discovery group is missing a valid versions array"))?;
        for version in versions {
            let fallback_version = version
                .get("version")
                .and_then(Value::as_str)
                .filter(|version| !version.is_empty())
                .ok_or_else(|| {
                    anyhow::anyhow!("discovery version is missing a non-empty version name")
                })?;
            let version_resources = version
                .get("resources")
                .and_then(Value::as_array)
                .ok_or_else(|| {
                    anyhow::anyhow!("discovery version is missing a valid resources array")
                })?;
            for resource in version_resources {
                let response_kind = resource
                    .get("responseKind")
                    .and_then(Value::as_object)
                    .ok_or_else(|| {
                        anyhow::anyhow!("discovery resource is missing a responseKind object")
                    })?;
                let kind = response_kind
                    .get("kind")
                    .and_then(Value::as_str)
                    .filter(|kind| !kind.is_empty())
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "discovery resource responseKind is missing a non-empty kind"
                        )
                    })?;
                let group = response_kind
                    .get("group")
                    .and_then(Value::as_str)
                    .filter(|group| !group.is_empty())
                    .unwrap_or(fallback_group);
                let version_name = response_kind
                    .get("version")
                    .and_then(Value::as_str)
                    .filter(|version| !version.is_empty())
                    .unwrap_or(fallback_version);
                let namespaced = match resource.get("scope").and_then(Value::as_str) {
                    Some("Namespaced") => true,
                    Some("Cluster") => false,
                    Some(scope) => bail!("unknown Kubernetes discovery scope: {scope}"),
                    None => bail!("Kubernetes discovery resource is missing `scope`"),
                };
                let key = format!("{}/{kind}", api_version(group, version_name));
                if resources.insert(key.clone(), namespaced).is_some() {
                    bail!("duplicate discovery entry for resource {key}");
                }
            }
        }
    }
    Ok(resources)
}

fn read_json(path: &Path) -> Result<Value> {
    let text =
        std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    serde_json::from_str(&text).with_context(|| format!("parsing {}", path.display()))
}

fn group_version_kinds(schema: &Value) -> Vec<&Map<String, Value>> {
    schema
        .get("x-kubernetes-group-version-kind")
        .and_then(Value::as_array)
        .map(|items| items.iter().filter_map(Value::as_object).collect())
        .unwrap_or_default()
}

fn api_version(group: &str, version: &str) -> String {
    if group.is_empty() {
        version.to_owned()
    } else {
        format!("{group}/{version}")
    }
}

fn collection_scope(document: &Value, group: &str, version: &str, kind: &str) -> Option<bool> {
    let prefix = if group.is_empty() {
        format!("/api/{version}/")
    } else {
        format!("/apis/{group}/{version}/")
    };
    for (path, item) in document.get("paths").and_then(Value::as_object)? {
        let Some(suffix) = path.strip_prefix(&prefix) else {
            continue;
        };
        let namespaced = match suffix.split('/').collect::<Vec<_>>().as_slice() {
            [resource] if !resource.is_empty() => false,
            ["namespaces", "{namespace}", resource] if !resource.is_empty() => true,
            _ => continue,
        };
        let Some(operations) = item.as_object() else {
            continue;
        };
        if operations.values().any(|operation| {
            let Some(operation_gvk) = operation
                .get("x-kubernetes-group-version-kind")
                .and_then(Value::as_object)
            else {
                return false;
            };
            operation_gvk
                .get("group")
                .and_then(Value::as_str)
                .unwrap_or("")
                == group
                && operation_gvk
                    .get("version")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    == version
                && operation_gvk
                    .get("kind")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    == kind
        }) {
            return Some(namespaced);
        }
    }
    None
}
