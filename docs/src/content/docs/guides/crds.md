---
title: Use custom resources
description: Generate typed Nix options from Kubernetes CustomResourceDefinitions.
---

Give Nixifest one or more CRD YAML files through `specialArgs.crds`. During evaluation it generates and imports typed Nix options for each served version.

CRD input may be a multi-document YAML file. Nixifest skips non-CRD documents. Each `CustomResourceDefinition` document must use `apiextensions.k8s.io/v1`, and every served version in its `spec.versions` must provide a schema at `schema.openAPIV3Schema`.

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

The generated options are available immediately in the same evaluation. Set `validation.strict = true` to validate custom resources against the generated schema.
