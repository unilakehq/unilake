namespace Unilake.Iac.Kubernetes.Helm.Input;

public sealed class OpenSearchArgs : HelmInputArgs
{
    public bool SingleNode { get; set; } = true;
    public override string Version { get; set; } = "2.16.1";
}