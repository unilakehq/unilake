using Pulumi;

namespace Unilake.Iac.Kubernetes.Helm.Input;

public sealed class KafkaInputArgs : HelmInputArgs
{
    public override string Version { get; set; } = "22.1.1";
    public string ImageTag { get; set; } = "3.4.0-debian-11-r28";
    public required Input<string> SchemaRegistryUrl { get; init; }
}