---
title: Resources
description: Map Kubernetes resources to Nix attributes.
---

Nixifest groups resources by group, version, kind, and name:

```text
resources.<group>.<version>.<kind>.<name>
```

For example, this defines a ConfigMap named `app`:

```nix
{ inputs, ... }:
{
  imports = [ inputs.nixifest.modules.nixifest.latest ];

  resources.core.v1.ConfigMap.app = {
    metadata.namespace = "default";
    data.MESSAGE = "hello";
  };
}
```

Nixifest adds `apiVersion`, `kind`, and `metadata.name` from the resource path. The `core` group maps to Kubernetes' ungrouped API versions. The resulting manifest contains:

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: app
  namespace: default
data:
  MESSAGE: hello
```

Set `metadata.name` explicitly when it should differ from the final path key. Set `validation.strict = true` to validate fields against the generated schema; see the [Resource options reference](/reference/resources/).

Null-valued attributes are removed before output. See [Build outputs](/reference/outputs/) for the generated manifest and YAML values.
