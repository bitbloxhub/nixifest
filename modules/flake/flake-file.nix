{
  inputs,
  ...
}:
{
  flake-file = {
    inputs = {
      flake-file.url = "github:vic/flake-file";
      flake-parts = {
        url = "github:hercules-ci/flake-parts";
        inputs.nixpkgs-lib.follows = "nixpkgs";
      };
    };
    outputs =
      # nix
      ''
        inputs:
        inputs.flake-parts.lib.mkFlake { inherit inputs; } (inputs.import-tree ./modules/flake)
      '';
  };

  imports = [
    inputs.flake-file.flakeModules.default
    inputs.flake-file.flakeModules.import-tree
  ];

  systems = [
    "x86_64-linux"
    "aarch64-linux"
    "aarch64-darwin"
  ];
}
