using Pulumi;

namespace Unilake.Iac.Kubernetes.Helm.Input;

public class DatahubArgs : HelmInputArgs
{
    public override string Version { get; set; } = "0.2.164";
    public Input<string> PostgreSqlHost { get; set; }
    public Input<string> PostgreSqlPort { get; set; }
    public Input<string> PostgreSqlDatabaseName { get; set; }
    public Input<string> PostgreSqlUsername { get; set; }
    public Input<bool> FrontendEnabled { get; set; }
    public Input<string> ElasticSearchHost { get; set; }
    public Input<int> ElasticSearchPort { get; set; }
    public Input<string>? ElasticSearchPrefix { get; set; } = null;
    public Input<string> KafkaBootstrapServer { get; set; }
    public Input<string> KafkaSchemaRegistryUrl { get; set; }
    public Input<string> PostgreSqlPassword { get; set; }
}