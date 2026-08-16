---
title: Modules
description: Select and import generated Kubernetes schema modules.
---

Nixifest publishes one module for each generated Kubernetes minor version under `inputs.nixifest.modules.nixifest`.

```nix
{ inputs, ... }:
{
  imports = [ inputs.nixifest.modules.nixifest.latest ];
}
```

## Select a version

Use `latest` to follow the newest generated schema. Pin a version when schema changes should be deliberate:

```nix
imports = [ inputs.nixifest.modules.nixifest.v1_36 ];
```

Versioned modules provide typed options for Kubernetes resources from that API schema. Custom-resource options are added separately when CRD files are passed through `specialArgs.crds`; see [Custom resources](/guides/crds/).
