---
title: Getting started
description: Build your first Kubernetes manifest with Nixifest.
---
Nixifest turns Kubernetes API schemas into typed Nix module options, then builds one YAML file from your resource definitions.

## 1. Define a manifest

Create `manifest.nix`:

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

The resource path identifies API version and kind. `hello` supplies the default name. See [Resources](/guides/resources/) for the mapping in detail.

## 2. Evaluate and package the result

Add Nixifest and Nixpkgs to `flake.nix`, then expose `config.build.yaml` as a package:

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    nixifest.url = "github:bitbloxhub/nixifest";
  };

  outputs = inputs@{ nixpkgs, nixifest, ... }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
    in {
      packages.${system}.manifest = (nixifest.lib.eval {
        specialArgs = { inherit pkgs inputs; };
        modules = [ ./manifest.nix ];
      }).config.build.yaml;
    };
}
```

`pkgs` supplies Nix build helpers. `inputs.nixifest.lib.eval` returns standard Nix module results; `config.build.yaml` is the generated multi-document YAML file.

Use `latest` for newest generated schema. Pin a version when reproducibility matters; see the [Modules reference](/reference/modules/).

## 3. Build and deploy

```console
$ nix build .#manifest
$ kubectl apply -f result
```

If you use GitOps, publish the YAML where your controller can fetch it—for example, as an OCI artifact referenced by Flux’s `OCIRepository`. See [Build outputs](/reference/outputs/) for other `config.build` values.
