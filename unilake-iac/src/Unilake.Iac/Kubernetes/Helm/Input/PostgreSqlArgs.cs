using Pulumi;

namespace Unilake.Iac.Kubernetes.Helm.Input;

public sealed class PostgreSqlArgs : HelmInputArgs
{
    public required Input<string> Username { get; set; }
    public required Input<string> Password { get; set; }
    public required string[]? Databases { get; set; }
    public Input<string> AppName { get; set; } = "general";
    public override string Version { get; set; } = "13.2.15";
}