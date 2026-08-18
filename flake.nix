# DO-NOT-EDIT. This file was auto-generated using github:vic/flake-file.
# Use `nix run .#write-flake` to regenerate it.
{
  outputs =
    inputs: inputs.flake-parts.lib.mkFlake { inherit inputs; } (inputs.import-tree ./modules/flake);

  inputs = {
    crate2nix = {
      url = "github:nix-community/crate2nix";
      inputs = {
        cachix.follows = "";
        flake-compat.follows = "";
        flake-parts.follows = "flake-parts";
        nixpkgs.follows = "nixpkgs";
      };
    };
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-file.url = "github:vic/flake-file";
    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };
    flint = {
      url = "github:NotAShelf/flint";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    github-actions-nix = {
      url = "github:synapdeck/github-actions-nix";
      inputs = {
        flake-parts.follows = "flake-parts";
        nixpkgs.follows = "nixpkgs";
      };
    };
    import-tree.url = "github:vic/import-tree";
    junix = {
      url = "gitlab:moduon/junix";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        precommix.follows = "precommix";
        systems.follows = "systems";
      };
    };
    make-shell = {
      url = "github:nicknovitski/make-shell";
      inputs.flake-compat.follows = "";
    };
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    nuschtos-search = {
      url = "github:NuschtOS/search";
      inputs = {
        flake-utils.inputs.systems.follows = "systems";
        nix-index-database.follows = "";
        nixpkgs.follows = "nixpkgs";
      };
    };
    precommix = {
      url = "gitlab:moduon/precommix/v0.36.0";
      inputs = {
        blueprint.inputs.systems.follows = "systems";
        devshell.follows = "crate2nix/devshell";
        nixpkgs.follows = "nixpkgs";
      };
    };
    systems.url = "github:nix-systems/triplet";
    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
}
