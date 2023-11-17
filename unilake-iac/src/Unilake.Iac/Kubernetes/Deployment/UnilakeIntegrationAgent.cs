using Pulumi;

namespace Unilake.Iac.Kubernetes.Deployment;

/// <summary>
/// Unilake internal integration agent deployment
/// </summary>
public class UnilakeIntegrationAgent : KubernetesComponentResource
{
    public UnilakeIntegrationAgent(string name, ComponentResourceOptions? options = null) 
        : base("unilake:kubernetes:deployment:unilake:integrationagent", name, options)
    {
        throw new NotImplementedException();
    }
}