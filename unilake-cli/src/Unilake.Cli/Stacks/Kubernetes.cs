using Pulumi.Kubernetes.Core.V1;
using Unilake.Cli.Config;
using Unilake.Iac;
using Unilake.Iac.Kubernetes;
using Unilake.Iac.Kubernetes.Custom;
using Unilake.Iac.Kubernetes.Deployment;
using Unilake.Iac.Kubernetes.Helm;
using Unilake.Iac.Kubernetes.Helm.Input;

namespace Unilake.Cli.Stacks;

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

        // Storage dependencies
        PostgreSql? postgreSql = CreatePostgreSqlInstance(kubernetesContext, @namespace);
        Redis? redis = CreateRedisInstance(kubernetesContext, @namespace);
        OpenSearch? openSearch = CreateOpenSearchInstance(kubernetesContext, @namespace);
        Kafka? kafka = CreateKafkaInstance(kubernetesContext, @namespace);
        Minio? minio = CreateMinioInstance(kubernetesContext, @namespace);

        // Service dependencies
        BoxyHq? boxyhq = CreateBoxyHqInstance(kubernetesContext, @namespace);
        Iac.Kubernetes.Helm.Datahub? datahub = CreateDatahubInstance(kubernetesContext, @namespace);
        StarRockCluster? starRockCluster = CreateStarRocksCluster(kubernetesContext, @namespace);
        Unilake.Iac.Kubernetes.Helm.Nessie? nessie = null;

        // Internal services
        UnilakeWeb? unilakeWeb = null;
        UnilakeApi? unilakeApi = null;
        UnilakeProxyQuery? unilakeProxyQuery = null;
        UnilakeProxyStorage? unilakeProxyStorage = null;

        // Development services
        Iac.Kubernetes.Helm.KafkaUi? kafkaUi = null;
        Iac.Kubernetes.Deployment.RedisUi? redisUi = null;
        Iac.Kubernetes.Helm.Gitea? gitea = null;
        PgWeb? pgWeb = null;

        throw new NotImplementedException();
    }

    private EnvironmentContext GetEnvironmentContext()
    {
        return new EnvironmentContext();
    }

    private Iac.Kubernetes.Helm.Nessie? CreateNessieInstance(KubernetesEnvironmentContext kubernetesEnvironmentContext, Namespace @namespace)
    {
        throw new NotImplementedException();
    }

    private StarRockCluster? CreateStarRocksCluster(KubernetesEnvironmentContext kubernetesEnvironmentContext, Namespace @namespace)
    {
        if(Config?.Components?.Starrocks?.Enabled ?? false)
        {
            //var starRocksClusterConfig = Config.Components.Starrocks;
            return new StarRockCluster(kubernetesEnvironmentContext, "");
        }
        return null;
    }

    private Minio? CreateMinioInstance(KubernetesEnvironmentContext kubernetesEnvironmentContext, Namespace @namespace)
    {
        if(Config.Cloud?.Kubernetes?.DataLake?.Minio?.Enabled ?? false)
        {
            var minioConfig = Config.Cloud.Kubernetes.DataLake.Minio;
            return new Minio(kubernetesEnvironmentContext, new MinioArgs
            {
                Replicas = minioConfig!.Replicas,
                RootUser = minioConfig.RootUser!,
                RootPassword = minioConfig.RootPassword!,
                Buckets = minioConfig.Buckets!.Select(x => new MinioArgsBucket
                {
                    Name = x.Name!,
                    ObjectLocking = x.ObjectLocking,
                    Policy = x.Policy!,
                    Purge = x.Purge,
                    Versioning = x.Versioning
                }).ToArray()
            }, @namespace);
        }
        return null;
    }

    private Kafka? CreateKafkaInstance(KubernetesEnvironmentContext kubernetesEnvironmentContext, Namespace @namespace)
    {
        if(Config.Cloud?.Kubernetes?.Kafka?.Enabled ?? false)
        {
            var kafkaConfig = Config.Cloud.Kubernetes.Kafka;
            return new Kafka(kubernetesEnvironmentContext, new KafkaInputArgs
            {
                SchemaRegistryUrl = kafkaConfig.SchemaRegistry!
            }, @namespace);
        }
        return null;
    }

    private OpenSearch? CreateOpenSearchInstance(KubernetesEnvironmentContext kubernetesEnvironmentContext, Namespace @namespace)
    {
        if(Config.Cloud?.Kubernetes?.Opensearch?.Enabled ?? false)
        {
            var openSearchConf = Config.Cloud.Kubernetes.Opensearch;
            return new OpenSearch(kubernetesEnvironmentContext, new OpenSearchArgs
            {
                SingleNode = openSearchConf.SingleNode
            }, @namespace);
        }
        return null;
    }

    private Redis? CreateRedisInstance(KubernetesEnvironmentContext kubernetesEnvironmentContext, Namespace @namespace)
    {
        if(Config.Cloud?.Kubernetes?.Redis?.Enabled ?? false)
        {
            //var _ = Config.Cloud.Kubernetes.Redis;
            return new Redis(kubernetesEnvironmentContext, new RedisArgs
            {
                // Nothing?
            }, @namespace);
        }

        return null;
    }

    private PostgreSql? CreatePostgreSqlInstance(KubernetesEnvironmentContext kubernetesEnvironmentContext, Namespace @namespace)
    {
        if(Config.Cloud?.Kubernetes?.Postgresql?.Enabled ?? false)
        {
            var postgresqlConf = Config.Cloud.Kubernetes.Postgresql;
            return new PostgreSql(kubernetesEnvironmentContext, new PostgreSqlArgs
            {
                AppName = "unilake",
                DatabaseName = "",
                Password = postgresqlConf.Password!,
                Username = postgresqlConf.Username!
            }, @namespace);
        }

        // TODO: check which databases need to be created and provide access to them (boxyhq, api, datahub, nessie etc.., or do in their respective functions to keep everything together?)

        return null;
    }

    private BoxyHq? CreateBoxyHqInstance(KubernetesEnvironmentContext kubernetesEnvironmentContext, Namespace @namespace)
    {
        if(Config.Components?.Boxyhq?.Enabled ?? false)
        {
            var boxyhqConf = Config.Components.Boxyhq.Postgresql;
            return new BoxyHq(kubernetesEnvironmentContext, new Iac.Kubernetes.Deployment.Input.BoxyHqInputArgs
            { 
                DbDatabaseName = boxyhqConf!.Schema!,
                DbEndpoint = boxyhqConf.Host!,
                DbEngine = "postgresql",
                DbPassword = boxyhqConf.Password!,
                DbUsername = boxyhqConf.Username!,
                DbPort = boxyhqConf.Port,
                JacksonApiKey = Array.Empty<string>()
            }, @namespace);
        }

        return null;
    }

    private Iac.Kubernetes.Helm.Datahub? CreateDatahubInstance(KubernetesEnvironmentContext kubernetesEnvironmentContext, Namespace @namespace)
    {
        if(Config.Components?.Datahub?.Enabled ?? false)
        {
            var datahubConfig = Config.Components.Datahub;
            return new Iac.Kubernetes.Helm.Datahub(kubernetesEnvironmentContext, new DatahubArgs
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
            }, @namespace);
        }

        return null;
    }
}
