{
  self,
  ...
}:
{
  perSystem =
    {
      pkgs,
      self',
      ...
    }:
    let
      basic = self.lib.eval {
        specialArgs = { inherit pkgs self; };
        modules = [ ../../modules/nixifest/examples/basic.nix ];
      };
      crd = pkgs.callPackage ../../modules/nixifest/examples/crd.nix {
        inherit self self';
      };
    in
    {
      packages.example_basic = basic.config.build.yaml;
      packages.example_crd = crd.config.build.yaml;
    };
}
