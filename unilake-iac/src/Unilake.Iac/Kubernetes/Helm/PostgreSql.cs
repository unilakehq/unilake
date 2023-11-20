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
        : base("unilake:kubernetes:helm:postgresql", name, options, checkNamingConvention)
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
        string[] databases = InputArgs?.Databases ?? Array.Empty<string>();
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
            Values = new InputMap<object> // https://github.com/bitnami/charts/blob/main/bitnami/postgresql/values.yaml
                    {
                        ["global"] = new Dictionary<string, object>
                        {
                            ["postgresql"] = new Dictionary<string, object>
                            {
                                ["auth"] = new Dictionary<string, object>
                                {
                                    ["username"] = inputArgs.Username,
                                    ["existingSecret"] = secret.Metadata.Apply(x => x.Name)
                                }
                            },
                            ["primary"] = new Dictionary<string, object>
                            {
                                ["initdb"] = new Dictionary<string, object>
                                {
                                    ["scripts"] = new InputMap<string>
                                    {
                                        {"dbs_init_script.sh", InitMultipleDatabasesScript}
                                    }
                                },
                                ["extraEnvVars"] = new List<object>
                                {
                                    new EnvVarArgs
                                    {
                                        Name = "POSTGRES_MULTIPLE_DATABASES",
                                        Value = string.Join(',', databases)
                                    }
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

    private string InitMultipleDatabasesScript => """
        #!/bin/bash

            set -e
            set -u

            function create_user_and_database() {
                local database=$1
                echo "  Creating user and database '$database'"
                psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" <<-EOSQL
                    CREATE USER $database;
                    CREATE DATABASE $database;
                    GRANT ALL PRIVILEGES ON DATABASE $database TO $database;
            EOSQL
            }

            if [ -n "$POSTGRES_MULTIPLE_DATABASES" ]; then
                echo "Multiple database creation requested: $POSTGRES_MULTIPLE_DATABASES"
                for db in $(echo $POSTGRES_MULTIPLE_DATABASES | tr ',' ' '); do
                    create_user_and_database $db
                done
                echo "Multiple databases created"
            fi
    """;
}