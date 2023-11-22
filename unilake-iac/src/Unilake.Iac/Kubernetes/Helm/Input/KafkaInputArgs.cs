using Pulumi;

namespace Unilake.Iac.Kubernetes.Helm.Input;

public class KafkaInputArgs : HelmInputArgs
{
    public override string Version { get; set; } = "26.4.2";
    public string ImageTag { get; set; } = "3.6.0-debian-11-r2";
    public required Input<string> SchemaRegistryUrl { get; init; }
}