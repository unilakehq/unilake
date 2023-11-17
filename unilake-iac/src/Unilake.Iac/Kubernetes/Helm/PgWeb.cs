using Pulumi;
using Pulumi.Kubernetes.Core.V1;
using Pulumi.Kubernetes.Helm.V3;
using Pulumi.Kubernetes.Types.Inputs.Core.V1;
using Pulumi.Kubernetes.Types.Inputs.Helm.V3;
using Pulumi.Kubernetes.Types.Inputs.Meta.V1;
using Unilake.Iac.Kubernetes.Helm.Input;

namespace Unilake.Iac.Kubernetes.Helm;

public class PgWeb : KubernetesComponentResource
{
    [Output("name")] 
    public Output<string>? Name { get; private set; }

    public Service? @Service { get; private set; }

    public PgWeb(KubernetesEnvironmentContext ctx, PostgreSql postgreSql, string databaseName, Namespace? @namespace = null,
        string name = "pgweb", ComponentResourceOptions? options = null, bool checkNamingConvention = true)
        : base("unilake:kubernetes:helm:pgweb", name, options, checkNamingConvention) => Create(ctx, new PgWebArgs
    {
        PgHost = postgreSql.Service.Spec.Apply(x => x.ClusterIP),
        PgPort = postgreSql.Service.Spec.Apply(x => x.Ports[0].Port),
        PgPassword = postgreSql.Secret.Data.Apply(x => x["password"]),
        PgUsername = postgreSql.Secret.Data.Apply(x => x["username"]),
        PgDatabase = databaseName,
    }, @namespace, name, options);

    public PgWeb(KubernetesEnvironmentContext ctx, PgWebArgs inputArgs, Namespace? @namespace = null,
        string name = "pgweb", ComponentResourceOptions? options = null, bool checkNamingConvention = true)
        : base("unilake:kubernetes:helm:pgweb", name, options, checkNamingConvention) =>
        Create(ctx, inputArgs, @namespace, name, options);

    private void Create(KubernetesEnvironmentContext ctx, PgWebArgs inputArgs, Namespace? @namespace = null,
        string name = "pgweb", ComponentResourceOptions? options = null)
    {
        // Set default options
        var resourceOptions = CreateOptions(options);
        resourceOptions.Parent = this;
        resourceOptions.Provider = ctx.Provider;

        // Set namespace
        @namespace = SetNamespace(resourceOptions, name, @namespace);

        // Create secret for authentication details
        var secret = new Secret(name, new SecretArgs
        {
            Metadata = new ObjectMetaArgs
            {
                Name = name,
                Namespace = @namespace.Metadata.Apply(x => x.Name),
            },
            Data = new InputMap<string>
            {
                {
                    "DATABASE_URL", Output.All(inputArgs.PgUsername.ToOutput(), inputArgs.PgPassword.ToOutput(),
                            inputArgs.PgHost.ToOutput(), inputArgs.PgPort.Apply(x => x.ToString()),
                            inputArgs.PgDatabase.ToOutput())
                        .Apply(x => $"postgres://{x[0]}:{x[1]}@{x[2]}:{x[3]}/{x[4]}?sslmode=disable".EncodeBase64())
                },
            }
        }, resourceOptions);

        //Get PgWeb chart and add
        var releaseArgs = new ReleaseArgs
        {
            Name = name,
            Chart = "pgweb",
            Version = inputArgs.Version,
            Namespace = @namespace.Metadata.Apply(x => x.Name),
            RepositoryOpts = new RepositoryOptsArgs
            {
                Repo = "https://charts.ectobit.com"
            },
            Values = new InputMap<object> // https://github.com/ectobit/charts/blob/main/pgweb/values.yaml
            {
                ["env"] = new List<object>
                {
                    new EnvVarArgs
                    {
                        Name = "DATABASE_URL",
                        ValueFrom = new EnvVarSourceArgs
                        {
                            SecretKeyRef = new SecretKeySelectorArgs
                            {
                                Name = name,
                                Key = "DATABASE_URL",
                            },
                        },
                    }
                }
            },
            // By default Release resource will wait till all created resources
            // are available. Set this to true to skip waiting on resources being
            // available.
            SkipAwait = false
        };

        // PgWeb instance
        resourceOptions.DependsOn = secret;
        var pgWebInstance = new Release(name, releaseArgs, resourceOptions);

        // Get output
        var status = pgWebInstance.Status;
        @Service = Service.Get(name, Output.All(status).Apply(s => $"{s[0].Namespace}/{s[0].Name}"),
            resourceOptions);
        Name = pgWebInstance.Name;
    }
}