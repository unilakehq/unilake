using Pulumi;
using Pulumi.Kubernetes.Core.V1;
using Pulumi.Kubernetes.Helm.V3;
using Pulumi.Kubernetes.Types.Inputs.Core.V1;
using Pulumi.Kubernetes.Types.Inputs.Helm.V3;
using Pulumi.Kubernetes.Types.Inputs.Meta.V1;
using Unilake.Iac.Kubernetes.Helm.Input;

namespace Unilake.Iac.Kubernetes.Helm;

public class Datahub : KubernetesComponentResource
{
    [Output("name")] 
    public Output<string> Name { get; private set; }

    public Service @Service { get; private set; }
    
    public Datahub(KubernetesEnvironmentContext ctx, DatahubArgs inputArgs, Namespace? @namespace = null, 
        string name = "datahub", ComponentResourceOptions? options = null, bool checkNamingConvention = true)
        : base("unilake:kubernetes:helm:datahub", name, options, checkNamingConvention)
    {
        // check input
        if (inputArgs == null)
            throw new ArgumentNullException(nameof(inputArgs), "inputArgs cannot be null");
        
        // Set default options
        var resourceOptions = CreateOptions(options);
        resourceOptions.Parent = this;
        resourceOptions.Provider = ctx.Provider;

        // Set namespace
        @namespace = SetNamespace(resourceOptions, name, @namespace);

        // Create secret for sql-authentication
        var secretName = name + "custom";
        var _secret = new Secret(secretName, new SecretArgs{
            Metadata = new ObjectMetaArgs{
                Name = secretName,
                Namespace = @namespace.Metadata.Apply(x => x.Name),
            },
            Data = new InputMap<string>
            {
                { "postgres-password", inputArgs.PostgreSqlPassword.Apply(x => x.EncodeBase64()) },
                { "opensearch-password", "admin".EncodeBase64() },
            }
        }, resourceOptions); 

        //Get Datahub chart and add
        var releaseArgs = new ReleaseArgs
        {
            Name = name,
            Chart = "datahub",
            Version = inputArgs.Version,
            Namespace = @namespace.Metadata.Apply(x => x.Name),
            RepositoryOpts = new RepositoryOptsArgs
            {
                Repo = "https://helm.datahubproject.io/"
            },
            // TODO: service loadbalancers are not needed (must be clusterip instead)
            // TODO: deamonsets are created with svclb?
            Values = new InputMap<object> // https://github.com/acryldata/datahub-helm/blob/master/charts/datahub/values.yaml
                    {
                        ["postgresqlSetupJob.enabled"] = true,
                        ["datahub-frontend.enabled"] = inputArgs.FrontendEnabled,
                        ["elasticsearchSetupJob.extraEnvs"] = new Input<InputMap<string>>[]
                        {
                            new InputMap<string>
                            {
                                ["name"] = "USE_AWS_ELASTICSEARCH",
                                ["value"] = "true"
                            }
                        },
                        ["mysqlSetupJob"] = new Dictionary<string, object>
                        {
                            ["enabled"] = false
                        },
                        ["postgresqlSetupJob"] = new Dictionary<string, object>
                        {
                            ["enabled"] = true
                        },
                        ["global"] = new Dictionary<string, object>
                        {
                            ["graph_service_impl"] = "elasticsearch",
                            ["datahub_analytics_enabled"] = false,
                            ["datahub_standalone_consumers_enabled"] = true,
                            ["elasticsearch"] = new Dictionary<string, object>
                            {
                                ["host"] = inputArgs.ElasticSearchHost,
                                ["port"] = inputArgs.ElasticSearchPort,
                                // TODO: prefix is not working yet
                                ["indexPrefix"] = inputArgs.ElasticSearchPrefix,
                                ["useSSL"] = "true",
                                ["skipcheck"] = "true",
                                ["insecure"] = "true", // TODO: Also needs to be enabled for mae consumer and gms
                                // TODO: set username and password from inputArgs
                                ["auth"] = new Dictionary<string, object>
                                {
                                    ["username"] = "admin",
                                    ["password"] = new Dictionary<string, object>
                                    {
                                        ["secretRef"] = secretName,
                                        ["secretKey"] = "opensearch-password",
                                    }
                                }
                            },
                            ["kafka"] = new Dictionary<string, object>
                            {
                                // TODO: for kafka topics, we will need to set a prefix, due to multi-tenant setup 
                                ["bootstrap"] = new Dictionary<string, object>
                                {
                                    ["server"] = inputArgs.KafkaBootstrapServer
                                },
                                ["zookeeper"] = new Dictionary<string, object>
                                {
                                    ["server"] = ""
                                },
                                ["schemaregistry"] = new Dictionary<string, object>
                                {
                                    ["url"] = inputArgs.KafkaSchemaRegistryUrl
                                }
                            },
                            ["sql"] = new Dictionary<string, object>
                            {
                                ["datasource"] = new Dictionary<string, object>
                                {
                                    ["hostForMysqlClient"] = "",
                                    ["hostForpostgresqlClient"] = inputArgs.PostgreSqlHost,
                                    ["host"] =  Output.All(inputArgs.PostgreSqlHost, inputArgs.PostgreSqlPort)
                                        .Apply(x => $"{x[0]}:{x[1]}"),
                                    ["port"] = inputArgs.PostgreSqlPort,
                                    ["url"] = Output.All(inputArgs.PostgreSqlHost, inputArgs.PostgreSqlPort, inputArgs.PostgreSqlDatabaseName)
                                        .Apply(x => $"jdbc:postgresql://{x[0]}:{x[1]}/{x[2]}"),
                                    ["driver"] = "org.postgresql.Driver",
                                    ["username"] = inputArgs.PostgreSqlUsername,
                                    ["password"] = new Dictionary<string, object>
                                    {
                                        ["secretRef"] = secretName,
                                        ["secretKey"] = "postgres-password"
                                    }
                                }
                            },
                            //Sub Charts
                            // TODO: not sure if this is working and the way to set subchart values, still getting a loadbalancer ip
                            ["datahub-frontend"] = new Dictionary<string, object>
                            {
                                ["service.type"] = "ClusterIp"
                            }
                        }
                    },
            // By default Release resource will wait till all created resources
            // are available. Set this to true to skip waiting on resources being
            // available.
            SkipAwait = false
        };

        // Check if a private registry is used
        if(inputArgs.UsePrivateRegsitry)
            throw new NotImplementedException("Private registry is currently not supported");

        // Datahub instance
        var datahubInstance = new Release(name, releaseArgs, resourceOptions);

        // Get output
        var status = datahubInstance.Status;
        @Service = Service.Get(name, Output.All(status).Apply(s => $"{s[0].Namespace}/{s[0].Name}"), resourceOptions);
        Name = datahubInstance.Name;
    }
}