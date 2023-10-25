using Pulumi;

namespace Unilake.Iac.Kubernetes.Deployment;

/// <summary>
/// Unilake internal query gateway deployment
/// </summary>
public class UnilakeProxyStorage : KubernetesComponentResource
{
    public UnilakeProxyStorage(string name, ComponentResourceOptions? options = null) 
        : base("pkg:kubernetes:deployment:unilake:proxystorage", name, options)
    {
        throw new NotImplementedException();
    }
}