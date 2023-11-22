using Pulumi;

namespace Unilake.Iac.Kubernetes.Deployment;

/// <summary>
/// Dbt runtime Unilake internal deployment
/// </summary>
public class UnilakeDbtRuntime : KubernetesComponentResource
{
    public UnilakeDbtRuntime(string name, ComponentResourceOptions? options = null) 
        : base("unilake:kubernetes:deployment:unilake:dbtruntime", name, options)
    {
        throw new NotImplementedException();
    }
}