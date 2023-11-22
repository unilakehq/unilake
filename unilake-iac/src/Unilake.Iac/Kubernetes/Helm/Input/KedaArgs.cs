namespace Unilake.Iac.Kubernetes.Helm.Input;

public sealed class KedaArgs : HelmInputArgs
{
    public override string Version { get; set; } = "2.12.0";
}