using Pulumi;
using Pulumi.Kubernetes.Core.V1;
using Pulumi.Kubernetes.Helm.V3;
using Pulumi.Kubernetes.Types.Inputs.Core.V1;
using Pulumi.Kubernetes.Types.Inputs.Helm.V3;
using Pulumi.Kubernetes.Types.Inputs.Meta.V1;
using Unilake.Iac.Kubernetes.Helm.Input;

namespace Unilake.Iac.Kubernetes.Helm;

public class Gitea : KubernetesComponentResource
{
    public Gitea(KubernetesEnvironmentContext ctx, GiteaArgs inputArgs, Namespace? @namespace = null, 
        string name = "gitea", ComponentResourceOptions? options = null, bool checkNamingConvention = true)
        : base("unilake:kubernetes:helm:gitea", name, options, checkNamingConvention)
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
                Name = name + "-admin",
                Namespace = @namespace.Metadata.Apply(x => x.Name),
            },           
            Data = new InputMap<string>
            {
                { "username", inputArgs.AdminUsername.Apply(x => x.EncodeBase64()) },
                { "password", inputArgs.AdminPassword.Apply(x => x.EncodeBase64()) },
                { "email", inputArgs.AdminEmail.Apply(x => x.EncodeBase64()) },
            }
        }, resourceOptions); 
        
        //Get Gitea chart and add
        var releaseArgs = new ReleaseArgs
        {
            Name = name,
            Chart = "gitea",
            Version = inputArgs.Version,
            Namespace = @namespace.Metadata.Apply(x => x.Name),
            RepositoryOpts = new RepositoryOptsArgs
            {
                Repo = "https://dl.gitea.io/charts/"
            },
            Values = new InputMap<object> // https://gitea.com/gitea/helm-chart/src/branch/main/values.yaml
            {
                ["gitea"] = new Dictionary<string, object>
                {
                    ["admin"] = new Dictionary<string, object>
                    {
                        ["existingSecret"] = secret.Metadata.Apply(x => x.Name),
                    },
                    ["config"] = new Dictionary<string, object> // https://docs.gitea.io/en-us/config-cheat-sheet/
                    {
                        ["database"] = new Dictionary<string, object>
                        {
                            ["DB_TYPE"] = "postgres",
                            ["HOST"] = inputArgs.PostgreSqlHost,
                            ["NAME"] = inputArgs.PostgreSqlDatabaseName,
                            ["USER"] = inputArgs.PostgreSqlUser,
                            ["PASSWD"] = inputArgs.PostgreSqlPassword,
                            ["SCHEMA"] = inputArgs.PostgreSqlSchemaName,
                        },
                        ["cache"] = new Dictionary<string, object>
                        {
                            ["ADAPTER"] = "redis",
                            ["HOST"] = Output.All(
                                inputArgs.RedisPassword.ToOutput(),
                                inputArgs.RedisHost.ToOutput(), 
                                inputArgs.RedisPort.Apply(x => x.ToString()), 
                                inputArgs.RedisDatabase.Apply(x => x.ToString()))
                                .Apply(x => $"redis://:{x[0]}@{x[1]}:{x[2]}/{x[3]}?pool_size=100&idle_timeout=180s")
                        },
                    }
                },
                ["postgresql"] = new Dictionary<string, object>
                {
                    ["enabled"] = false
                },
                ["memcached"] = new Dictionary<string, object>
                {
                    ["enabled"] = false
                },
                ["resources"] = new Dictionary<string, object>
                {
                    ["limits"] = new Dictionary<string, object>
                    {
                        ["cpu"] = "250m",
                        ["memory"] = "512Mi"
                    },
                    ["requests"] = new Dictionary<string, object>
                    {
                        ["cpu"] = "100m",
                        ["memory"] = "128Mi"
                    }
                }
            },
            // By default Release resource will wait till all created resources
            // are available. Set this to true to skip waiting on resources being
            // available.
            SkipAwait = false
        };

        // Create instance
        resourceOptions.DependsOn = secret;
        var giteaInstance = new Release(name, releaseArgs, resourceOptions);

        // Get output
        var _ = giteaInstance.Status;
    }
}