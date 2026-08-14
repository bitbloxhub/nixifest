{
  lib,
  baseModule,
}:
{
  modules ? [ ],
  specialArgs ? { },
}:
let
  inherit (specialArgs) pkgs;
  nixifest-typegen = (pkgs.callPackage ../nixifest-typegen/Cargo.nix { }).rootCrate.build;
in
lib.evalModules {
  class = "nixifest";
  modules = [
    baseModule
  ]
  ++ modules;
  specialArgs = {
    crds = [ ];
    inherit nixifest-typegen;
  }
  // specialArgs;
}
