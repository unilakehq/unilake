using Unilake.Cli.Config;
using Unilake.Iac;
using Unilake.Iac.Kubernetes;
using Unilake.Iac.Kubernetes.Deployment;
using Unilake.Iac.Kubernetes.Helm;

namespace Unilake.Cli;

public sealed class Kubernetes : UnilakeStack
{
    public Kubernetes(EnvironmentConfig config) : base(config)
    {
    }

    public override (string name, string version)[] Packages => 
        new [] {("kubernetes", "v4.5.3")};

    public override Task Create()
    {
        var context = GetEnvironmentContext();
        var kubernetesContext = KubernetesEnvironmentContext.Create(context, "");
        var @namespace = kubernetesContext.GetNamespace("default");

        BoxyHQ? boxyhq;
        Iac.Kubernetes.Helm.Datahub? datahub;

        if(Config.Components?.Boxyhq?.Enabled ?? false)
        {
            boxyhq = new BoxyHQ(kubernetesContext, @namespace, new Iac.Kubernetes.Deployment.Input.BoxyHqInputArgs
            { 
                DbDatabaseName = Config.Components.Boxyhq.Postgresql!.Schema!,
                DbEndpoint = Config.Components.Boxyhq.Postgresql.Host!,
                DbEngine = "postgresql",
                DbPassword = Config.Components.Boxyhq.Postgresql.Password!,
                DbUsername = Config.Components.Boxyhq.Postgresql.Username!,
                DbPort = Config.Components.Boxyhq.Postgresql.Port,
            });
        }

        if(Config.Components?.Datahub?.Enabled ?? false)
        {
            var datahubConfig = Config.Components.Datahub;
            datahub = new Iac.Kubernetes.Helm.Datahub(kubernetesContext, @namespace, new Iac.Kubernetes.Helm.Input.DatahubArgs
            {
                PostgreSqlDatabaseName = datahubConfig.Postgresql!.Schema!,
                PostgreSqlHost = datahubConfig.Postgresql.Host!,
                PostgreSqlPort = datahubConfig.Postgresql.Port.ToString(),
                PostgreSqlUsername = datahubConfig.Postgresql.Username!,
                PostgreSqlPassword = datahubConfig.Postgresql.Password!,

                ElasticSearchHost = datahubConfig.Opensearch!.Host!,
                ElasticSearchPort = datahubConfig.Opensearch.Port,
                ElasticSearchPrefix = "",

                FrontendEnabled = false,

                KafkaBootstrapServer = datahubConfig.Kafka!.Server!,
                KafkaSchemaRegistryUrl = datahubConfig.Kafka.SchemaRegistry!,
            });
        }


        throw new NotImplementedException();
    }

    private EnvironmentContext GetEnvironmentContext()
    {
        return new EnvironmentContext();

    }
}
