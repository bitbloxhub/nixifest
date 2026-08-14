{
  inputs,
  ...
}:
{
  imports = [
    inputs.flake-parts.flakeModules.modules
  ];

  flake.modules.nixifest = {
    default = ../../modules/nixifest;
  }
  // (import ../../modules/nixifest/generated);
}
