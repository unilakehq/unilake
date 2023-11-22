using Pulumi;

namespace Unilake.Iac.Kubernetes.Deployment;

/// <summary>
/// Unilake dbt ide deployment
/// </summary>
public class UnilakeDbtIde : KubernetesComponentResource
{
    public UnilakeDbtIde(string name, ComponentResourceOptions? options = null) 
        : base("unilake:kubernetes:deployment:unilake:dbtide", name, options)
    {
        throw new NotImplementedException();
    }
}