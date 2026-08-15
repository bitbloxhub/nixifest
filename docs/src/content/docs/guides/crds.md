---
title: Use custom resources
description: Generate typed Nix options from Kubernetes CustomResourceDefinitions.
---

Nixifest can read a CRD, generate a Nix module for its versions, and add that module to your evaluation.

## Provide CRD files

Pass CRD YAML files through `specialArgs.crds`. The CRD generator runs during evaluation and produces typed resource options.

```nix
{ inputs, pkgs, ... }:

inputs.nixifest.lib.eval {
  specialArgs = {
    inherit pkgs inputs;
    crds = [ ./widget-crd.yaml ];
  };

  modules = [
    {
      imports = [ inputs.nixifest.modules.nixifest.latest ];

      resources."example.com/v1".Widget.demo = {
        metadata.namespace = "default";
        spec.size = 3;
      };
    }
  ];
};
```

The example uses a local CRD file. For a remote CRD, replace the path with a pinned `fetchurl` expression containing its URL and hash.
