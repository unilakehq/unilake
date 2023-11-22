using Pulumi;

namespace Unilake.Iac.Kubernetes.Deployment;

/// <summary>
/// Unilake public website
/// </summary>
public class UnilakeWww : KubernetesComponentResource
{
    public UnilakeWww(string name, ComponentResourceOptions? options = null) 
        : base("unilake:kubernetes:deployment:unilake:www", name, options)
    {
        throw new NotImplementedException();
    }
}