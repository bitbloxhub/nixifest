---
title: Define resources
description: Define typed Kubernetes resources with Nixifest.
---

Resources are grouped by API version, kind, and logical name. The logical name supplies a default `metadata.name`, while explicit metadata takes precedence.

```nix
{ inputs, ... }:
{
  imports = [ inputs.nixifest.modules.nixifest.latest ];

  validation.strict = true;

  resources."apps/v1".Deployment.web = {
    metadata.namespace = "default";
    metadata.name = "frontend";

    spec = {
      replicas = 2;
      selector.matchLabels.app = "frontend";
      template = {
        metadata.labels.app = "frontend";
        spec.containers = [
          {
            name = "nginx";
            image = "nginx:1.31-alpine";
          }
        ];
      };
    };
  };
}
```

During build evaluation, Nixifest recursively removes null-valued attributes, adds `apiVersion` and `kind`, and exposes the resulting manifests through `config.build.manifests` and `config.build.yaml`.

## Validate resources

`validation.strict = true` applies generated schema validation to every resource: built-in Kubernetes APIs and custom resources. It rejects unknown fields and validates generated field types, enums, and other schema constraints during Nix evaluation.

With strict mode disabled, Nixifest accepts free-form resource attributes. That can help with unsupported or rapidly changing APIs, but moves validation to Kubernetes after YAML generation.
