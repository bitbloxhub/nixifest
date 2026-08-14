{
  perSystem =
    {
      pkgs,
      ...
    }:
    {
      make-shells.default.packages = [
        pkgs.nixfmt
        pkgs.deadnix
        pkgs.statix
      ];

      treefmt.programs = {
        deadnix.enable = true;
        nixfmt.enable = true;
        statix.enable = true;
      };
    };
}
