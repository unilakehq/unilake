using Pulumi;
using Pulumi.Kubernetes.Core.V1;
using Pulumi.Kubernetes.Helm.V3;
using Pulumi.Kubernetes.Types.Inputs.Helm.V3;
using Unilake.Iac.Kubernetes.Helm.Input;

namespace Unilake.Iac.Kubernetes.Helm;

public class Redis : KubernetesComponentResource
{
    [Output("password")] 
    public Output<string> Password { get; private set; }
    
    [Output("name")] 
    public Output<string> Name { get; private set; }

    public Service @Service { get; private set; }
    
    public Redis(KubernetesEnvironmentContext ctx, Namespace? @namespace = null, RedisArgs? inputArgs = null, 
        string name = "redis", ComponentResourceOptions? options = null, bool checkNamingConvention = true)
        : base("pkg:kubernetes:helm:redis", name, options, checkNamingConvention)
    {
        // Check input
        inputArgs ??= new RedisArgs();

        // Set default options
        var resourceOptions = CreateOptions(options);
        resourceOptions.Parent = this;
        resourceOptions.Provider = ctx.Provider;

        // Set namespace
        @namespace = SetNamespace(resourceOptions, name, @namespace);

        //Get Redis chart and add
        var releaseArgs = new ReleaseArgs
        {
            Name = name,
            Chart = "redis",
            Version = inputArgs.Version,
            Namespace = @namespace.Metadata.Apply(x => x.Name),
            RepositoryOpts = new RepositoryOptsArgs
            {
                Repo = "https://charts.bitnami.com/bitnami"
            },
            Values =
                new
                    InputMap<object> // https://github.com/bitnami/charts/blob/main/bitnami/redis/values.yaml
                    {
                        // ["master"] = new Dictionary<string, object>
                        // {
                        //     ["podLabels"] = GetLabels(ctx, inputArgs.AppName, inputArgs.AppName, "redis", inputArgs.Version)
                        // },
                        // ["replica"] = new Dictionary<string, object>
                        // {
                        //     ["podLabels"] = GetLabels(ctx, inputArgs.AppName, inputArgs.AppName, "redis", inputArgs.Version)
                        // },
                        ["architecture"] = "standalone",
                        //["commonLabels"] = GetLabels(ctx, inputArgs.AppName, null, "redis", inputArgs.Version)
                    },
            // By default Release resource will wait till all created resources
            // are available. Set this to true to skip waiting on resources being
            // available.
            SkipAwait = false
        };

        // Check if a private registry is used
        if(inputArgs.UsePrivateRegsitry)
        {
            var registrySecret = CreateRegistrySecret(ctx, resourceOptions, @namespace.Metadata.Apply(x => x.Name));
            string PrivateRegistryBase = !string.IsNullOrWhiteSpace(inputArgs.PrivateRegistryBase) ? inputArgs.PrivateRegistryBase + "/" : "";
            releaseArgs.Values.Add("global.imageRegistry", PrivateRegistryBase);
            releaseArgs.Values.Add("global.imagePullSecrets", new [] {registrySecret.Metadata.Apply(x => x.Name)});
        }

        // Redis instance
        var redisInstance = new Release(name, releaseArgs, resourceOptions);

        // Get output
        var status = redisInstance.Status;
        var secret = Secret.Get(name + "-secret", Output.All(status).Apply(c => $"{c[0].Namespace}/{c[0].Name}"), resourceOptions);
        Password = Output.CreateSecret(secret.Data.Apply(x =>  x["redis-password"].DecodeBase64()));
        @Service = Service.Get(name, Output.All(status).Apply(s => $"{s[0].Namespace}/{s[0].Name}-master"), resourceOptions);
        Name = redisInstance.Name;
    }
}