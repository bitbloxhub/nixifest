{
  crds,
  lib,
  pkgs,
  nixifest-typegen,
  ...
}:
let
  generated = map (
    crd:
    (pkgs.runCommand "nixifest-crd-types.nix"
      {
        nativeBuildInputs = [ nixifest-typegen ];
      }
      ''
        nixifest-typegen crd \
          --input ${toString crd} \
          --output "$out"
      ''
    ).outPath
  ) crds;
in
{
  imports = generated;

  options.build.crds = lib.mkOption {
    readOnly = true;
    type = lib.types.listOf lib.types.package;
  };

  config.build.crds = generated;
}
