using Pulumi;

namespace Unilake.Iac.Kubernetes.Helm.Input;

public sealed class GiteaArgs : HelmInputArgs
{
    public required Input<string> AdminUsername { get; set; }
    public required Input<string> AdminPassword { get; set; }
    public required Input<string> AdminEmail { get; set; }
    public required Input<string> PostgreSqlHost { get; set; }
    public required Input<string> PostgreSqlDatabaseName { get; set; }
    public required Input<string> PostgreSqlUser { get; set; }
    public required Input<string> PostgreSqlPassword { get; set; }
    public required Input<string> PostgreSqlSchemaName { get; set; }
    public override string Version { get; set; } = "8.1.0";
    public Input<string> RedisPassword { get; set; } = string.Empty;
    public required Input<string> RedisHost { get; set; }
    public required Input<int> RedisPort { get; set; } = 6379;
    public Input<int> RedisDatabase { get; set; } = 0;
}