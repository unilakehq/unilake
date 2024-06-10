using Pulumi;

namespace Unilake.Iac.Kubernetes.Resource.Input;

public class ConfigMapArgs
{
    public required Input<string> Namespace { get; set; }
    public required InputMap<string> Data { get; set; }
}
