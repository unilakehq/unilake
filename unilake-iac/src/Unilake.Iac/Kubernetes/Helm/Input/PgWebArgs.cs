using Pulumi;

namespace Unilake.Iac.Kubernetes.Helm.Input;

public class PgWebArgs : HelmInputArgs
{
    public Input<string> PgUsername { get; set; }
    public Input<string> PgPassword { get; set; }
    public Input<string> PgHost { get; set; }
    public Input<int> PgPort { get; set; }
    public Input<string> PgDatabase { get; set; }
    public override string Version { get; set; } = "0.1.7";
}