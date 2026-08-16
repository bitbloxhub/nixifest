{
  self,
  ...
}:
{
  flake-file.inputs.nuschtos-search = {
    url = "github:NuschtOS/search";
    # inputs.flake-utils.follows = "flake-utils";
    # inputs.ixx.follows = "ixx"
    inputs.nix-index-database.follows = "";
    inputs.nixpkgs.follows = "nixpkgs";
  };

  perSystem =
    {
      pkgs,
      inputs',
      ...
    }:
    {
      packages.search = inputs'.nuschtos-search.packages.mkSearch {
        optionsJSON = "${
          (pkgs.nixosOptionsDoc {
            inherit
              (
                (self.lib.eval {
                  modules = [
                    {
                      imports = [ self.modules.nixifest.latest ];
                      validation.strict = true;
                    }
                  ];
                })
              )
              options
              ;
            warningsAreErrors = false;
          }).optionsJSON
        }/share/doc/nixos/options.json";
        baseHref = "/options/search/";
        title = "Nixifest Options";
        urlPrefix = "https://github.com/bitbloxhub/nixifest/blob/main/";
      };
    };
}
