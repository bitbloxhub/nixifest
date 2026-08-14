{
  lib,
  config,
  ...
}:
let
  inherit (lib) mkEnableOption mkOption types;

  looseResource = types.attrsOf types.anything;
in
{
  options = {
    resources = mkOption {
      default = { };
      description = "Kubernetes resources.";
      type = types.submodule {
        freeformType =
          if config.validation.strict then
            null
          else
            types.attrsOf (types.attrsOf (types.attrsOf looseResource));
      };
    };
    validation.strict = mkEnableOption "strict Kubernetes resource typing";
  };
}
