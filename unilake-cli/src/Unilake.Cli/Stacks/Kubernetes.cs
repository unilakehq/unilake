using OneOf;
using OneOf.Types;
using Pulumi;
using Pulumi.Kubernetes.Core.V1;
using Unilake.Cli.Config;
using Unilake.Iac;
using Unilake.Iac.Kubernetes;
using Unilake.Iac.Kubernetes.Custom;
using Unilake.Iac.Kubernetes.Deployment;
using Unilake.Iac.Kubernetes.Deployment.Input;
using Unilake.Iac.Kubernetes.Helm;
using Unilake.Iac.Kubernetes.Helm.Input;

namespace Unilake.Cli.Stacks;

internal sealed class Kubernetes : UnilakeStack
{
    public Kubernetes(EnvironmentConfig config) : base(config)
    {
    }

    public override (string name, string version)[] Packages =>
        new[] { ("kubernetes", "v4.5.3") };

    public override Task<OneOf<Success, Error<Exception>>> Create()
    {
        var context = GetEnvironmentContext();
        var kubernetesContext = KubernetesEnvironmentContext.Create(context, "some-context");
        var @namespace = kubernetesContext.GetNamespace("default");

        // Storage dependencies
        PostgreSql? postgreSql = CreatePostgreSqlInstance(kubernetesContext, @namespace);
        Redis? redis = CreateRedisInstance(kubernetesContext, @namespace);
        OpenSearch? openSearch = CreateOpenSearchInstance(kubernetesContext, @namespace);
        Kafka? kafka = CreateKafkaInstance(kubernetesContext, @namespace);
        Minio? minio = CreateMinioInstance(kubernetesContext, @namespace);

        // Service dependencies
        BoxyHq? boxyhq = CreateBoxyHqInstance(kubernetesContext, @namespace, postgreSql);
        // Datahub? datahub = CreateDatahubInstance(kubernetesContext, @namespace, postgreSql, openSearch, kafka);
        // TODO: fix this
        // StarRockCluster? starRockCluster = CreateStarRocksCluster(kubernetesContext, @namespace);
        // Nessie? nessie = CreateNessieInstance(kubernetesContext, @namespace, postgreSql);
        //
        // // Internal services
        // UnilakeWeb? unilakeWeb = null;
        // UnilakeApi? unilakeApi = null;
        // UnilakeProxyQuery? unilakeProxyQuery = null;
        // UnilakeProxyStorage? unilakeProxyStorage = null;
        //

        // Development services
        KafkaUi? kafkaUi = CreateKafkaUiInstance(kubernetesContext, @namespace, kafka);
        RedisUi? redisUi = CreateRedisUiInstance(kubernetesContext, @namespace, redis);
        // TODO: fix this
        // Gitea? gitea = null;
        PgWeb? pgWeb = CreatePgWebInstance(kubernetesContext, @namespace, postgreSql);

        return Task.FromResult(new OneOf<Success, Error<Exception>>());
    }

    private EnvironmentContext GetEnvironmentContext() => new EnvironmentContext();

    private bool IsDevelopmentEnabled() => Config.Components?.Development?.Enabled ?? false;

    private PgWeb? CreatePgWebInstance(KubernetesEnvironmentContext kubernetesEnvironmentContext, Namespace @namespace, PostgreSql? postgreSqlInstance)
    {
        if (IsDevelopmentEnabled() && postgreSqlInstance != null && (Config.Components?.Development?.Enabled ?? false))
        {
            var targetDatabase = Config.Components.Development.Pgweb?.Database ?? "postgres";
            return new PgWeb(kubernetesEnvironmentContext, postgreSqlInstance, targetDatabase, @namespace);
        }

        return null;
    }

    private RedisUi? CreateRedisUiInstance(KubernetesEnvironmentContext kubernetesEnvironmentContext,
        Namespace @namespace, Redis? redisInstance)
    {
        if (IsDevelopmentEnabled() && redisInstance != null &&
            (Config.Components?.Development?.RedisUi?.Enabled ?? false))
            return new RedisUi(kubernetesEnvironmentContext, redisInstance, new RedisUiArgs
            {
                // nothing ??
            }, @namespace);
        return null;
    }

    private KafkaUi? CreateKafkaUiInstance(KubernetesEnvironmentContext kubernetesEnvironmentContext, Namespace @namespace, Kafka? kafkaInstance)
    {
        if (IsDevelopmentEnabled() && kafkaInstance != null && (Config.Components?.Development?.KafkaUi?.Enabled ?? false))
        {
            var bootstrapAddress = Output.Tuple(kafkaInstance.Name, kafkaInstance.Service.Metadata).Apply(x => $"{x.Item1}-0.kafka-headless.{x.Item2.Namespace}.svc.cluster.local:9092");
            return new KafkaUi(kubernetesEnvironmentContext, new KafkaUiInputArgs
            {
                ServerName = kafkaInstance.Name,
                ServerBootstrapAddress = bootstrapAddress
            }, @namespace);
        }
        return null;
    }

    private Nessie? CreateNessieInstance(KubernetesEnvironmentContext kubernetesEnvironmentContext, Namespace @namespace, PostgreSql? postgreSql)
    {
        throw new NotImplementedException();
    }

    private StarRockCluster? CreateStarRocksCluster(KubernetesEnvironmentContext kubernetesEnvironmentContext, Namespace @namespace)
    {
        if (Config.Components?.Starrocks?.Enabled ?? false)
        {
            //var starRocksClusterConfig = Config.Components.Starrocks;
            return new StarRockCluster(kubernetesEnvironmentContext, "");
        }
        return null;
    }

    private Minio? CreateMinioInstance(KubernetesEnvironmentContext kubernetesEnvironmentContext, Namespace @namespace)
    {
        if (Config.Cloud?.Kubernetes?.DataLake?.Minio?.Enabled ?? false)
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
        if (Config.Cloud?.Kubernetes?.Kafka?.Enabled ?? false)
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
        if (Config.Cloud?.Kubernetes?.Opensearch?.Enabled ?? false)
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
        if (Config.Cloud?.Kubernetes?.Redis?.Enabled ?? false)
        {
            return new Redis(kubernetesEnvironmentContext, new RedisArgs
            {
                // nothing ??
            }, @namespace);
        }

        return null;
    }

    private PostgreSql? CreatePostgreSqlInstance(KubernetesEnvironmentContext kubernetesEnvironmentContext, Namespace @namespace)
    {
        if (Config.Cloud?.Kubernetes?.Postgresql?.Enabled ?? false)
        {
            var postgresqlConf = Config.Cloud.Kubernetes.Postgresql;
            // check for all databses to be created at init
            var databases = new[]
            {
                Config.Components?.Nessie?.Postgresql?.Name ?? string.Empty,
                Config.Components?.Datahub?.Postgresql?.Name ?? string.Empty,
                Config.Components?.Boxyhq?.Postgresql?.Name ?? string.Empty,
                Config.Components?.Development?.Gitea?.Postgresql?.Name ?? string.Empty
            }.Where(x => !string.IsNullOrEmpty(x)).ToArray();

            return new PostgreSql(kubernetesEnvironmentContext, new PostgreSqlArgs
            {
                AppName = "unilake",
                Password = postgresqlConf.Password!,
                Username = postgresqlConf.Username!,
                Databases = databases
            }, @namespace);
        }
        return null;
    }

    private BoxyHq? CreateBoxyHqInstance(KubernetesEnvironmentContext kubernetesEnvironmentContext, Namespace @namespace, PostgreSql? postgreSql = null)
    {
        if (Config.Components?.Boxyhq?.Enabled ?? false)
        {
            var boxyhqConf = Config.Components.Boxyhq.Postgresql;
            Output<string> host = Output.Create(boxyhqConf?.Host!);
            if (postgreSql == null)
                throw new CliException("Cannot initialize postgresql for BoxyHQ as it does not exist");
            else if (String.Equals(boxyhqConf?.Host!, "cloud.kubernetes", StringComparison.InvariantCultureIgnoreCase))
                host = postgreSql.Service.Spec.Apply(x => x.ClusterIP);
            return new BoxyHq(kubernetesEnvironmentContext, new BoxyHqInputArgs
            {
                DbDatabaseName = boxyhqConf!.Name!,
                DbEndpoint = host!,
                DbPassword = boxyhqConf.Password!,
                DbUsername = boxyhqConf.Username!,
                DbPort = boxyhqConf.Port,
                JacksonApiKey = Array.Empty<string>()
            }, @namespace);
        }

        return null;
    }

    private Datahub? CreateDatahubInstance(KubernetesEnvironmentContext kubernetesEnvironmentContext, Namespace @namespace,
        PostgreSql? postgreSql = null, OpenSearch? openSearch = null, Kafka? kafka = null)
    {
        if (Config.Components?.Datahub?.Enabled ?? false)
        {
            var datahubConfig = Config.Components.Datahub!;

            // postgresql
            Output<string> postgreSqlHost = postgreSql == null ? Output.Create(datahubConfig.Postgresql!.Host!) : postgreSql!.Service.Metadata.Apply(x => x.Name);
            Output<string> postgreSqlPort = postgreSql == null ? Output.Create(datahubConfig.Postgresql!.Port.ToString()) : postgreSql!.Service.Spec.Apply(x => x.Ports[0].Port.ToString());
            Output<string> postgreSqlUsername = postgreSql == null ? Output.Create(datahubConfig.Postgresql!.Username!) : postgreSql!.Secret.Data.Apply(x => x["username"].DecodeBase64());
            Output<string> postgreSqlPassword = postgreSql == null ? Output.Create(datahubConfig.Postgresql!.Password!) : postgreSql!.Secret.Data.Apply(x => x["password"].DecodeBase64());

            // opensearch
            Output<string> openSearchHost = openSearch == null ? Output.Create(datahubConfig.Opensearch!.Host!) :
                openSearch!.Service.Metadata.Apply(x => x.Name);
            Output<int> openSearchPort = openSearch == null ? Output.Create(datahubConfig.Opensearch!.Port) :
                openSearch!.Service.Spec.Apply(x => x.Ports[0].Port);

            // kafka
            Output<string> kafkaHost = kafka == null ? Output.Create(datahubConfig.Kafka!.Server!) :
                Output.Tuple(kafka!.Service.Metadata, kafka!.Service.Spec).Apply(x => $"{x.Item1.Name}:{x.Item2.Ports[0].Port}");

            return new Datahub(kubernetesEnvironmentContext, new DatahubArgs
            {
                PostgreSqlDatabaseName = datahubConfig.Postgresql!.Name!,
                PostgreSqlHost = postgreSqlHost,
                PostgreSqlPort = postgreSqlPort,
                PostgreSqlUsername = postgreSqlUsername,
                PostgreSqlPassword = postgreSqlPassword,

                ElasticSearchHost = openSearchHost,
                ElasticSearchPort = openSearchPort,
                ElasticSearchPrefix = "",

                FrontendEnabled = false,

                KafkaBootstrapServer = kafkaHost,
            }, @namespace);
        }

        return null;
    }
}
