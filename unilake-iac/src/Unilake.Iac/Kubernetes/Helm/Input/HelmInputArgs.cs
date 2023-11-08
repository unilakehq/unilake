namespace Unilake.Iac.Kubernetes.Helm.Input;

public abstract class HelmInputArgs : BaseInputArgs
{
    public abstract string Version { get; set; }
}