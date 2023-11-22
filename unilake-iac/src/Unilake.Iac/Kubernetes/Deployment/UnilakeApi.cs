using Pulumi;

namespace Unilake.Iac.Kubernetes.Deployment;

/// <summary>
/// Unilake webapi
/// </summary>
public sealed class UnilakeApi : KubernetesComponentResource
{
    public UnilakeApi(string name, ComponentResourceOptions? options = null) 
        : base("unilake:kubernetes:deployment:unilake:webapi", name, options)
    {
        throw new NotImplementedException();
    }
}