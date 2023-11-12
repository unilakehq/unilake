using Pulumi;
using Pulumi.Kubernetes.Core.V1;
using Pulumi.Kubernetes.Helm.V3;
using Pulumi.Kubernetes.Types.Inputs.Core.V1;
using Pulumi.Kubernetes.Types.Inputs.Helm.V3;
using Pulumi.Kubernetes.Types.Inputs.Meta.V1;
using Unilake.Iac.Kubernetes.Helm.Input;

namespace Unilake.Iac.Kubernetes.Helm;

/// <summary>
/// See: https://github.com/bitnami/charts/tree/main/bitnami/postgresql
/// </summary>
public class PostgreSql : KubernetesComponentResource
{
    public PostgreSqlArgs InputArgs { get; private set; }
    
    [Output("name")] 
    public Output<string> Name { get; private set; }

    public Service @Service { get; private set; }

    public Secret @Secret { get; private set; }

    public PostgreSql(KubernetesEnvironmentContext ctx, PostgreSqlArgs inputArgs, Namespace? @namespace = null, 
        string name = "postgresql", ComponentResourceOptions? options = null, bool checkNamingConvention = true)
        : base("pkg:kubernetes:helm:postgresql", name, options, checkNamingConvention)
    {
        // check input
        if (inputArgs == null) throw new ArgumentNullException(nameof(inputArgs));
        
        // Set default options
        var resourceOptions = CreateOptions(options);
        resourceOptions.Parent = this;
        resourceOptions.Provider = ctx.Provider;

        // Set namespace
        @namespace = SetNamespace(resourceOptions, name, @namespace);

        // Create secret for authentication details
        var secret = new Secret(name, new SecretArgs{
            Metadata = new ObjectMetaArgs{
                Name = name,
                Namespace = @namespace.Metadata.Apply(x => x.Name),
            },           
            Data = new InputMap<string>
            {
                { "postgres-password", inputArgs.Password.Apply(x => x.EncodeBase64()) },
                { "password", inputArgs.Password.Apply(x => x.EncodeBase64()) },
                { "replication-password", inputArgs.Password.Apply(x => x.EncodeBase64()) },
                { "username", inputArgs.Username.Apply(x => x.EncodeBase64()) },
            }
        }, resourceOptions); 

        //Get PostgreSql chart and add
        var releaseArgs = new ReleaseArgs
        {
            Name = name,
            Chart = "postgresql",
            Version = inputArgs.Version,
            Namespace = @namespace.Metadata.Apply(x => x.Name),
            RepositoryOpts = new RepositoryOptsArgs
            {
                Repo = "https://charts.bitnami.com/bitnami"
            },
            Values =
                new
                    InputMap<object> // https://github.com/bitnami/charts/blob/main/bitnami/postgresql/values.yaml
                    {
                        ["global"] = new Dictionary<string, object>
                        {
                            ["postgresql"] = new Dictionary<string, object>
                            {
                                ["auth"] = new Dictionary<string, object>
                                {
                                    ["username"] = inputArgs.Username,
                                    ["database"] = inputArgs.DatabaseName,
                                    ["existingSecret"] = secret.Metadata.Apply(x => x.Name)
                                }
                            }
                        },
                        //["commonLabels"] = GetLabels(ctx, inputArgs.AppName, null, "postgresql", inputArgs.Version)
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
            string privateRegistryBase = !string.IsNullOrWhiteSpace(inputArgs.PrivateRegistryBase) ? inputArgs.PrivateRegistryBase + "/" : "";
            releaseArgs.Values.Add("global.imageRegistry", privateRegistryBase);
            releaseArgs.Values.Add("global.imagePullSecrets", new [] {registrySecret.Metadata.Apply(x => x.Name)});
        }

        // PostgreSql instance
        var postgreInstance = new Release(name, releaseArgs, resourceOptions);

        // Get output
        var status = postgreInstance.Status;
        Service = Service.Get(name, Output.All(status).Apply(s => $"{s[0].Namespace}/{s[0].Name}"), resourceOptions);
        InputArgs = inputArgs;
        Name = postgreInstance.Name;
        Secret = secret;
    }
}