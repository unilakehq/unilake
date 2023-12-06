using Pulumi;
using Pulumi.Kubernetes.Types.Inputs.Meta.V1;
using Unilake.Iac.Kubernetes.Resource.Input;

namespace Unilake.Iac.Kubernetes.Resource;

public class ConfigMap : KubernetesComponentResource
{

    [Output("name")]
    public Output<string> Name { get; private set; }

    public ConfigMap(string name, ConfigMapArgs inputArgs, ComponentResourceOptions? options = null, bool checkNamingConvention = true)
        : base("unilake:kubernetes:resource:configmap", name, options, checkNamingConvention)
    {
        // set default options
        var resourceOptions = CreateOptions(options);
        resourceOptions.Parent = this;

        // set args
        var args = new Pulumi.Kubernetes.Types.Inputs.Core.V1.ConfigMapArgs
        {
            Metadata = new ObjectMetaArgs
            {
                Name = name,
                Namespace = inputArgs.Namespace
            },
            Data = inputArgs.Data,
        };

        // create resource
        var ConfigMap = new Pulumi.Kubernetes.Core.V1.ConfigMap(name, args, resourceOptions);
        Name = ConfigMap.Metadata.Apply(x => x.Name);

    }
}
