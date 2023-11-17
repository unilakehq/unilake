using Pulumi;

namespace Unilake.Iac.Kubernetes.Deployment;

/// <summary>
/// Unilake webapp Open Source
/// </summary>
public class UnilakeWeb : KubernetesComponentResource
{
    public UnilakeWeb(string name, ComponentResourceOptions? options = null) 
        : base("unilake:kubernetes:deployment:unilake:web", name, options)
    {
        throw new NotImplementedException();
    }
}