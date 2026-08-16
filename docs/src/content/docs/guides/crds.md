---
title: Use custom resources
description: Generate typed Nix options from Kubernetes CustomResourceDefinitions.
---

Pass Nixifest's typegen package through `specialArgs` so any module can generate and import typed options from CRD YAML files.

CRD input may be a multi-document YAML file. Nixifest skips non-CRD documents. Each `CustomResourceDefinition` document must use `apiextensions.k8s.io/v1`, and every served version in `spec.versions` must provide a schema at `schema.openAPIV3Schema`.

```nix
{ inputs, pkgs, ... }:
let
  nixifest-typegen = inputs.nixifest.packages.${pkgs.stdenv.hostPlatform.system}.typegen;
in
inputs.nixifest.lib.eval {
  specialArgs = {
    inherit pkgs nixifest-typegen;
  };

  modules = [
    inputs.nixifest.modules.nixifest.latest
    ./widgets.nix
  ];
}
```

Then, `widgets.nix` can import CRD schemas without direct access to flake inputs:

```nix
{ nixifest-typegen, ... }:
{
  imports = [
    (nixifest-typegen.importCRDs ./widget-crd.yaml)
  ];

  resources."example.com/v1".Widget.demo = {
    metadata.namespace = "default";
    spec.size = 3;
  };
}
```

`importCRDs` generates an importable Nix module containing typed options for every served CRD version. Passing `nixifest-typegen` through `specialArgs` makes the helper available to any module in the evaluation.

For remote CRDs, pass a pinned `fetchurl` path containing its URL and hash.

Set `validation.strict = true` to validate custom resources against the generated schema.
