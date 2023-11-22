using Pulumi;

namespace Unilake.Iac.Kubernetes.Deployment;

/// <summary>
/// Unilake public documentation site
/// </summary>
public sealed class UnilakeDocs : KubernetesComponentResource
{
    public UnilakeDocs(string name, ComponentResourceOptions? options = null, bool checkNamingConvention = true) 
        : base("unilake:kubernetes:deployment:unilake:docs", name, options, checkNamingConvention)
    {
        throw new NotImplementedException();
    }
}