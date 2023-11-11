using Pulumi;

namespace Unilake.Iac.Kubernetes.Helm.Input;

public class DatahubArgs : HelmInputArgs
{
    public override string Version { get; set; } = "0.2.164";
    public required Input<string> PostgreSqlHost { get; set; }
    public required Input<string> PostgreSqlPort { get; set; }
    public required Input<string> PostgreSqlDatabaseName { get; set; }
    public required Input<string> PostgreSqlUsername { get; set; }
    public required Input<bool> FrontendEnabled { get; set; }
    public required Input<string> ElasticSearchHost { get; set; }
    public required Input<int> ElasticSearchPort { get; set; }
    public Input<string> ElasticSearchPrefix { get; set; } = string.Empty;
    public required Input<string> KafkaBootstrapServer { get; set; }
    public required Input<string> KafkaSchemaRegistryUrl { get; set; }
    public required Input<string> PostgreSqlPassword { get; set; }
}