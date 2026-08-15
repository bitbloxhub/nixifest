---
title: Getting started
description: Build your first Kubernetes manifest with Nixifest.
---

Nixifest is a Nix module. Add it as a flake input, import a generated Kubernetes module, define resources, then evaluate `config.build.yaml`.

## 1. Add Nixifest to your flake

```nix
inputs.nixifest.url = "github:bitbloxhub/nixifest";
```

Create `manifest.nix` in consuming flake:

```nix
{ inputs, ... }:
{
  imports = [ inputs.nixifest.modules.nixifest.latest ];

  validation.strict = true;

  resources."apps/v1".Deployment.hello = {
    metadata.namespace = "default";
    spec = {
      replicas = 1;
      selector.matchLabels.app = "hello";
      template = {
        metadata.labels.app = "hello";
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

`latest` points to newest generated Kubernetes minor. Pin `v1_36` when reproducible schema selection matters.


## 2. Evaluate the module

Evaluate the module from your flake or another Nix expression:

```nix
inputs.nixifest.lib.eval {
  specialArgs = { inherit pkgs inputs; };
  modules = [ ./manifest.nix ];
}
```

`inputs.nixifest.lib.eval` returns standard Nix module results, including `config`, `options`, and the build outputs.


## 3. Build YAML

Expose the result as a flake package:

```nix
packages.manifest = (inputs.nixifest.lib.eval {
  specialArgs = { inherit pkgs inputs; };
  modules = [ ./manifest.nix ];
}).config.build.yaml;
```

Build it with `nix build .#manifest`. The output is a multi-document YAML file suitable for `kubectl apply -f`.
