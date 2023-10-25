namespace Unilake.Iac.Kubernetes.Helm.Input;

public class RedisArgs : HelmInputArgs
{
    public string? AppName { get; set; } = "general";

    public override string Version { get; set; } = "17.10.1";
}