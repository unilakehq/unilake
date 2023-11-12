using Pulumi;
using Pulumi.Kubernetes.Core.V1;
using Pulumi.Kubernetes.Helm.V3;
using Pulumi.Kubernetes.Types.Inputs.Core.V1;
using Pulumi.Kubernetes.Types.Inputs.Helm.V3;
using Pulumi.Kubernetes.Types.Inputs.Meta.V1;
using Unilake.Iac.Kubernetes.Helm.Input;

namespace Unilake.Iac.Kubernetes.Helm;

public class Nessie : KubernetesComponentResource
{
    [Output("name")] 
    public Output<string> Name { get; private set; }

    public Service @Service { get; private set; }

    
    public Nessie(KubernetesEnvironmentContext ctx, NessieArgs inputArgs, Namespace? @namespace = null, string name = "nessie", ComponentResourceOptions? options = null, bool checkNamingConvention = true) 
        : base("pkg:kubernetes:helm:nessie", name, options, checkNamingConvention)
    {
        // check input
        if (inputArgs == null) throw new ArgumentNullException(nameof(inputArgs));
        if (inputArgs is { StoreType: "postgresql", PosgreSqlUsername: null, PostgreSqlPassword: null })
            throw new ArgumentException("Must specify both PosgreSqlUsername and PostgreSqlPassword when store type is PostgreSQL");

        // set default options
        var resourceOptions = CreateOptions(options);
        resourceOptions.Parent = this;
        resourceOptions.Provider = ctx.Provider;
        
        // set namespace
        @namespace = SetNamespace(resourceOptions, name, @namespace);
        
        //get Nessie chart and add
        var releaseArgs = new ReleaseArgs
        {
            Name = name,
            Chart = "nessie",
            Version = inputArgs.Version,
            Namespace = @namespace.Metadata.Apply(x => x.Name),
            RepositoryOpts = new RepositoryOptsArgs
            {
                Repo = "https://charts.projectnessie.org"
            },
            Values =
                new
                    InputMap<object> // https://github.com/projectnessie/nessie/blob/main/helm/nessie/values.yaml
                    {
                        {"replicaCount", inputArgs.ReplicaCount},
                        {"versionStoreType", GetStoreType(inputArgs.StoreType)},
                        {
                            "image", new InputMap<string>
                            {
                                {"tag", inputArgs.ImageTag}
                            }
                        }
                    },
            // By default Release resource will wait till all created resources
            // are available. Set this to true to skip waiting on resources being
            // available.
            SkipAwait = false
        };
        
        // check for store details
        if (inputArgs is { StoreType: "postgresql", PosgreSqlUsername: not null, PostgreSqlPassword: not null })
        {
            // Create secret for authentication details
            var secret = new Secret(name, new SecretArgs{
                Metadata = new ObjectMetaArgs{
                    Name = name,
                    Namespace = @namespace.Metadata.Apply(x => x.Name),
                },           
                Data = new InputMap<string>
                {
                    { "postgres_username", inputArgs.PosgreSqlUsername },
                    { "postgres_password", inputArgs.PostgreSqlPassword },
                }
            }, resourceOptions); 
            
            // Set jdbc details
            releaseArgs.Values.Add("storeType", new InputMap<object>
            {
                {"jdbcUrl", inputArgs.PostgreSqlConnectionString },
                {
                    "secret", new InputMap<string>
                    {
                        {"name", secret.Metadata.Apply(x => x.Name)},
                        {"username", "postgres_username"},
                        {"password", "postgres_password"},
                    }
                },
            });
        }
        
        // Check if a private registry is used
        if(inputArgs.UsePrivateRegsitry)
        {
            throw new NotImplementedException();
        }
        
        // Nessie instance
        var nessieInstance = new Release(name, releaseArgs, resourceOptions);
        
        // Get output
        var status = nessieInstance.Status;
        Service = Service.Get(name, Output.All(status).Apply(s => $"{s[0].Namespace}/{s[0].Name}"), resourceOptions);
        Name = nessieInstance.Name;
    }

    private static string GetStoreType(string storeType) => storeType.ToLowerInvariant() switch
    {
        "postgresql" => "JDBC",
        _ => "IN_MEMORY"
    };
}