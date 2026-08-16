{
  fetchurl,
  pkgs,
  self,
  self',
  ...
}:
let
  nixifest-typegen = self'.packages.typegen;
in
self.lib.eval {
  specialArgs = { inherit pkgs; };
  modules = [
    {
      imports = [
        self.modules.nixifest.v1_36
        (nixifest-typegen.importCRDs (fetchurl {
          url = "https://github.com/cert-manager/cert-manager/releases/download/v1.21.1/cert-manager.crds.yaml";
          hash = "sha256-oTg74Chi3SgC5oEPGd0fR4RVDzNv9OsneI6EIwL3aXc=";
        }))
      ];

      validation.strict = true;

      resources."cert-manager.io/v1".Issuer.letsencrypt = {
        metadata.name = "letsencrypt";
        spec.acme = {
          server = "https://acme-v02.api.letsencrypt.org/directory";
          email = "user@example.com";
          profile = "tlsserver";
          privateKeySecretRef.name = "letsencrypt";
          solvers = [
            {
              http01.ingress.ingressClassName = "nginx";
            }
          ];
        };
      };
    }
  ];
}
