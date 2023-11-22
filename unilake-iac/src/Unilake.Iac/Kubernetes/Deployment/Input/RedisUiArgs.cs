namespace Unilake.Iac.Kubernetes.Deployment.Input;

public sealed class RedisUiArgs
{
    public string ServiceType { get; set; } = "ClusterIP";
}