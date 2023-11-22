using Pulumi;

namespace Unilake.Iac.Kubernetes.Helm.Input;

public sealed class PgWebArgs : HelmInputArgs
{
    public required Input<string> PgUsername { get; set; }
    public required Input<string> PgPassword { get; set; }
    public required Input<string> PgHost { get; set; }
    public Input<int> PgPort { get; set; } = 5432;
    public required Input<string> PgDatabase { get; set; }
    public override string Version { get; set; } = "0.1.9";
}