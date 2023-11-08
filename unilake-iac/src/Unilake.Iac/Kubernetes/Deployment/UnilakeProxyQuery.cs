using Pulumi;

namespace Unilake.Iac.Kubernetes.Deployment;

/// <summary>
/// Unilake internal query gateway deployment
/// </summary>
public class UnilakeProxyQuery : KubernetesComponentResource
{
    public UnilakeProxyQuery(string name, ComponentResourceOptions? options = null) 
        : base("pkg:kubernetes:deployment:unilake:proxyquery", name, options)
    {
        throw new NotImplementedException();
    }
}