namespace Unilake.Iac.Kubernetes.Deployment.Input;

public sealed class KarapaceInputArgs
{
    public required string KafkaServiceName { get; init; }
    public required int KafkaServicePort { get; init; }
}