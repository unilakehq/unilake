using Pulumi;

namespace Unilake.Iac.Kubernetes.Deployment;

/// <summary>
/// Unilake internal storage proxy deployment
/// </summary>
public sealed class UnilakeProxyStorage : KubernetesComponentResource
{
    public UnilakeProxyStorage(string name, ComponentResourceOptions? options = null) 
        : base("unilake:kubernetes:deployment:unilake:proxystorage", name, options)
    {
        throw new NotImplementedException();
    }
}