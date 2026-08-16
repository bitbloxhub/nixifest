{
  flake-file.inputs = {
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
  };

  perSystem =
    {
      pkgs,
      inputs',
      ...
    }:
    let
      typegenCargoNix = import ../../nixifest-typegen/Cargo.nix;

      typegenCargoWorkspace = pkgs.callPackage typegenCargoNix {
        buildRustCrateForPkgs =
          pkgs:
          with pkgs;
          buildRustCrate.override {
            cargo = inputs'.fenix.packages.default.toolchain;
            rustc = inputs'.fenix.packages.default.toolchain;
          };
      };
    in
    {
      make-shells.default.packages = [
        inputs'.fenix.packages.default.toolchain
        pkgs.rust-analyzer
        inputs'.crate2nix.packages.default
      ];

      packages.typegen = typegenCargoWorkspace.rootCrate.build.overrideAttrs (old: {
        passthru = (old.passthru or { }) // {
          importCRDs =
            crd:
            (pkgs.runCommand "nixifest-crd-types.nix"
              {
                nativeBuildInputs = [ typegenCargoWorkspace.rootCrate.build ];
              }
              ''
                nixifest-typegen crd \
                  --input ${toString crd} \
                  --output "$out"
              ''
            ).outPath;
        };
      });

      treefmt = {
        programs.rustfmt = {
          enable = true;
          package = inputs'.fenix.packages.default.rustfmt;
        };
        settings.global.excludes = [
          "**/Cargo.nix"
        ];
      };
    };
}
