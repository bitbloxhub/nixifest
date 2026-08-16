---
title: Build outputs
description: Outputs produced by Nixifest evaluations.
---
An evaluation returns build outputs under `config.build`:

- `config.build.manifests` — list of evaluated Kubernetes manifest attribute sets.
- `config.build.yaml` — one file containing every manifest as a multi-document YAML stream. Use it with `kubectl apply -f` or a GitOps system such as Flux.
- `config.build.crds` — generated Nix packages containing module-system definitions for supplied CustomResourceDefinitions.

Use `config.build.yaml` as a flake package or consume it through another Nix workflow. The [Getting started guide](/guides/getting-started/) shows complete flake wiring.
