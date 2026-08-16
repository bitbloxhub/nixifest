{
  lib,
  baseModule,
}:
{
  modules ? [ ],
  specialArgs ? { },
}:

lib.evalModules {
  class = "nixifest";
  modules = [
    baseModule
  ]
  ++ modules;
  inherit specialArgs;
}
