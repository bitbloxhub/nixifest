{
  perSystem =
    {
      pkgs,
      self',
      ...
    }:
    {
      make-shells.default = {
        packages = [
          pkgs.nodejs_latest
          pkgs.pnpm_11
        ];

        shellHook = ''
          export PATH=$PATH:$(pwd)/docs/node_modules/.bin/
        '';
      };

      packages.docs = pkgs.stdenv.mkDerivation (finalAttrs: {
        pname = "nixifest-docs";
        version = "0.0.1";
        src = ../../docs;
        pnpmDeps = pkgs.fetchPnpmDeps {
          fetcherVersion = 4;
          inherit (finalAttrs) pname version src;
          pnpm = pkgs.pnpm_11;
          hash = "sha256-djfso5aK3Wvj8MFTe08VPW0vT0JPP+viGU349FFwUYA=";
        };
        nativeBuildInputs = [
          pkgs.nodejs_latest
          pkgs.pnpm_11
          pkgs.pnpmConfigHook
        ];
        env.NUSCHT_SEARCH_PATH = self'.packages.search;
        buildPhase = ''
          pnpm run build
        '';
        installPhase = ''
          mkdir -p $out
          cp -r dist $out/
        '';
      });
    };
}
