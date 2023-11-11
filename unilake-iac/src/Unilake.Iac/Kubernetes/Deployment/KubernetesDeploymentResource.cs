using Pulumi;
using Pulumi.Kubernetes.Core.V1;
using Pulumi.Kubernetes.Types.Inputs.Core.V1;
using Pulumi.Kubernetes.Types.Inputs.Meta.V1;

namespace Unilake.Iac.Kubernetes.Deployment;

public abstract class KubernetesDeploymentResource : KubernetesComponentResource
{
    protected KubernetesDeploymentResource(string type, string name, ComponentResourceOptions? options = null, bool checkNamingConvention = true) : base(type, name, options, checkNamingConvention)
    {
    }

    protected KubernetesDeploymentResource(string type, string name, ResourceArgs? args, ComponentResourceOptions? options = null, bool remote = false) : base(type, name, args, options, remote)
    {
    }
    
    protected Secret CreateSecret(Namespace @namespace, string name, Dictionary<string, string> labels, InputMap<string> data, CustomResourceOptions resourceOptions) 
        => new (name, new SecretArgs
        {
            ApiVersion = "v1",
            Kind = "Secret",
            Metadata = new ObjectMetaArgs
            {
                Name = name,
                Namespace = @namespace.Metadata.Apply(x => x.Name),
                //Labels = labels
            },
            Type = "Opaque",
            Data = data
        }, resourceOptions);
}