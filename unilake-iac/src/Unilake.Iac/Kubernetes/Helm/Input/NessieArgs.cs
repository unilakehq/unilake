using Pulumi;

namespace Unilake.Iac.Kubernetes.Helm.Input;

public class NessieArgs : HelmInputArgs
{
    public override string Version { get; set; } = "0.58.1";
    public string StoreType { get; set; } = "IN_MEMORY";
    public required Input<string> PosgreSqlUsername { get; set; }
    public required Input<string> PostgreSqlPassword { get; set; }
    public required Input<string> PostgreSqlConnectionString { get; set; }
    public Input<int> ReplicaCount { get; set; } = 1;
    public Input<string> ImageTag { get; set; } = "0.58.1";
}