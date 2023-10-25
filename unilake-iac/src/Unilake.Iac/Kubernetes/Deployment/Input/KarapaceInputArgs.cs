namespace Unilake.Iac.Kubernetes.Deployment.Input;

public class KarapaceInputArgs
{
    public required string KafkaServiceName { get; init; }
    public required int KafkaServicePort { get; init; }
}