using Pulumi;

namespace Unilake.Iac.Kubernetes.Deployment;

/// <summary>
/// Unilake webapp Open Source
/// </summary>
public sealed class UnilakeWebApp : KubernetesComponentResource
{
    public UnilakeWebApp(string name, ComponentResourceOptions? options = null, bool checkNamingConvention = true) 
        : base("unilake:kubernetes:deployment:unilake:webapp", name, options, checkNamingConvention)
    {
        throw new NotImplementedException();
    }
}