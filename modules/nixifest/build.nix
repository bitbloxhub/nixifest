{
  lib,
  config,
  options,
  pkgs,
  ...
}:
let
  removeNulls =
    value:
    if lib.isAttrs value then
      lib.filterAttrs (_: item: item != null) (lib.mapAttrs (_: removeNulls) value)
    else if lib.isList value then
      map removeNulls value
    else
      value;
  yaml = pkgs.formats.yaml { };
in
{
  config.build = {
    manifests = lib.flatten (
      lib.mapAttrsToList (
        apiVersion: kinds:
        lib.mapAttrsToList (
          kind: resources:
          lib.mapAttrsToList (
            name: resource:
            let
              resource' = removeNulls resource;
            in
            resource'
            // {
              inherit apiVersion kind;
              metadata = {
                inherit name;
              }
              // (resource'.metadata or { });
            }
          ) resources
        ) kinds
      ) config.resources
    );
    yaml =
      let
        documents = map (
          manifest:
          yaml.generate "${lib.replaceStrings [ "/" ] [ "-" ] manifest.apiVersion}-${manifest.kind}-${manifest.metadata.name}.yaml" manifest
        ) config.build.manifests;
      in
      pkgs.runCommand "nixifest.yaml" { } ''
        : > "$out"

        first=1
        for document in ${lib.escapeShellArgs (map toString documents)}; do
          if [ "$first" -eq 0 ]; then
            printf '%s\n' '---' >> "$out"
          fi

          cat "$document" >> "$out"
          first=0
        done
      ''
      // {
        passthru = {
          inherit config options;
        };
      };
  };

  options.build = {
    manifests = lib.mkOption {
      description = "Flattened list of generated Kubernetes manifests.";
      readOnly = true;
      type = lib.types.listOf lib.types.attrs;
    };
    yaml = lib.mkOption {
      description = "Generated multi-document YAML package containing all manifests.";
      readOnly = true;
      type = lib.types.package;
    };
  };
}
