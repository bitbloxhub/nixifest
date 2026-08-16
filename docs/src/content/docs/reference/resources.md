---
title: Resource options
description: Resource paths, names, metadata, and validation.
---

Resources use this option path:

```text
resources.<apiVersion>.<kind>.<name>
```

Each segment has a role:

- `<apiVersion>` selects the Kubernetes API, such as `v1` or `apps/v1`.
- `<kind>` selects the resource kind, such as `ConfigMap` or `Deployment`.
- `<name>` identifies the resource in the Nix module and supplies `metadata.name` by default.

Set `metadata.name` explicitly to override the final path key. Other metadata, such as `metadata.namespace`, is passed through to the generated manifest.

## Validation

```nix
validation.strict = true;
```

Strict validation checks built-in resources and generated CRD resources against their schemas during Nix evaluation. It rejects unknown fields and invalid types, enums, and other schema constraints.

With strict validation disabled, resource attributes remain free-form. This can support APIs not covered by the selected schema, but validation happens later when Kubernetes receives the YAML.

Nixifest adds `apiVersion`, `kind`, and default metadata, removes null-valued attributes, and exposes the results through `config.build.manifests` and `config.build.yaml`. See [Resources](/guides/resources/) for a compact example.
