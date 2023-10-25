using Pulumi;

namespace Unilake.Iac.Kubernetes.Helm.Input;

public class GiteaArgs : HelmInputArgs
{
    public Input<string> AdminUsername { get; set; }
    public Input<string> AdminPassword { get; set; }
    public Input<string> AdminEmail { get; set; }
    public Input<string> PostgreSqlHost { get; set; }
    public Input<string> PostgreSqlDatabaseName { get; set; }
    public Input<string> PostgreSqlUser { get; set; }
    public Input<string> PostgreSqlPassword { get; set; }
    public Input<string> PostgreSqlSchemaName { get; set; }
    public override string Version { get; set; } = "8.1.0";
    
    public Input<string> RedisPassword { get; set; }
    public Input<string> RedisHost { get; set; }
    public Input<int> RedisPort { get; set; }
    public Input<int> RedisDatabase { get; set; }
}