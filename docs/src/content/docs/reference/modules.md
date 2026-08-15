---
title: Module reference
description: Core Nixifest modules and outputs.
---

## Generated Kubernetes modules

`inputs.nixifest.modules.nixifest.v1_27` through newest generated version expose typed Kubernetes resource options. `inputs.nixifest.modules.nixifest.latest` points to newest generated module.

Pin a version for reproducible schema behavior. Use `latest` when tracking newest Kubernetes release is intentional.

## Resource options

- `resources.<apiVersion>.<kind>.<name>`: define one resource.
- `metadata.name`: overrides the logical resource name when set.
- `validation.strict`: apply generated schema validation to built-in Kubernetes resources and CRDs.

Strict mode rejects unknown fields and validates generated types and schema constraints during evaluation. Without it, resource attributes remain free-form.

## Build outputs

- `build.manifests`: read-only list of evaluated Kubernetes manifests.
- `build.yaml`: read-only package containing all manifests as a multi-document YAML file.
- `build.crds`: read-only list of generated CRD type packages.

YAML document filenames use `<apiVersion>-<kind>-<metadata.name>.yaml`.
