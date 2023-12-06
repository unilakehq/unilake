using Pulumi;

namespace Unilake.Iac.Kubernetes.Deployment.Input;

public sealed class KarapaceInputArgs
{
    public required string KafkaServiceName { get; init; }
    public int KafkaServicePort { get; init; } = 9092;
    public string ImageVersion { get; set; } = "3.10.0";
}