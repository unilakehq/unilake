using Pulumi;

namespace Unilake.Iac.Kubernetes.Helm.Input;

public class KafkaUiInputArgs : HelmInputArgs
{
    public override string Version { get; set; } = "0.6.2";
    public required Input<string> ServerName { get; set; }
    public required Input<string> ServerBootstrapAddress { get; set; }
}