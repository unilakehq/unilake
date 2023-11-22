namespace Unilake.Iac.Kubernetes.Helm.Input;

public sealed class RedisArgs : HelmInputArgs
{
    public string? AppName { get; set; } = "general";
    public override string Version { get; set; } = "18.4.0";
}