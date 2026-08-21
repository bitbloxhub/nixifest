# Nixifest

Example:
```nix
# manifest.nix
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

```nix
# flake.nix
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

For more, see [the documentation](https://bitbloxhub.github.io/nixifest/).
