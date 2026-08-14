{
  self,
  ...
}:
{
  imports = [ self.modules.nixifest.v1_36 ];

  validation.strict = true;

  resources."apps/v1".Deployment.hello = {
    metadata.namespace = "default";

    spec = {
      replicas = 1;

      selector.matchLabels.app = "hello";

      template = {
        metadata.labels.app = "hello";

        spec.containers = [
          {
            image = "docker.io/library/nginx:1.31.3-alpine";
            name = "nginx";
            ports = [
              {
                containerPort = 80;
                protocol = "TCP";
              }
            ];
          }
        ];
      };
    };
  };
}
