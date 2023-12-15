using Pulumi;
using Pulumi.Kubernetes.Core.V1;
using Pulumi.Kubernetes.Types.Inputs.Meta.V1;
using Pulumi.Kubernetes.Types.Inputs.Networking.V1;

namespace Unilake.Iac.Kubernetes.Resource;

public class Ingress : KubernetesComponentResource
{
    [Output("name")]
    public Output<string> Name { get; private set; }

    public Ingress(EnvironmentContext ctx, string name, IngressSpecArgs ingressSpecArgs, Namespace? @namespace = null, bool skipAwait = false,
        Input<string>? clusterIssuer = null, Dictionary<string, string>? annotations = null, ComponentResourceOptions? options = null, bool checkNamingConvention = true)
        : base("unilake:kubernetes:resource:ingress", name, options, checkNamingConvention)
    {
        // set default options
        var resourceOptions = CreateOptions(options);
        resourceOptions.Parent = this;

        // set default annotations
        annotations ??= new Dictionary<string, string>();
        annotations.Add("pulumi.com/skipAwait", skipAwait ? "true" : "false");

        // Check for the labels
        var labels = clusterIssuer != null
            ? new InputMap<string>
            {
                { "cert-manager.io/cluster-issuer", clusterIssuer }
            }
            : new InputMap<string>();

        // Set args
        var args = new IngressArgs
        {
            Metadata = new ObjectMetaArgs
            {
                Name = name,
                Namespace = SetNamespace(resourceOptions, name, @namespace).Metadata.Apply(x => x.Name),
                Labels = labels,
                Annotations = annotations
            },
            Spec = ingressSpecArgs,
        };

        // Create resource
        var ingress = new global::Pulumi.Kubernetes.Networking.V1.Ingress(name, args, resourceOptions);

        // Set output
        Name = ingress.Metadata.Apply(x => x.Name);
    }

    public static IngressRuleArgs CreateServiceIngressRule(Input<string> host, Input<string> path, Input<string> pathtype,
        Input<string> serviceName, Input<string> portName)
        => new()
        {
            Host = host,
            Http = new HTTPIngressRuleValueArgs
            {
                Paths = new List<HTTPIngressPathArgs>
                {
                    new()
                    {
                        Path = path,
                        PathType = pathtype,
                        Backend = new IngressBackendArgs
                        {
                            Service = new IngressServiceBackendArgs
                            {
                                Name = serviceName,
                                Port = new ServiceBackendPortArgs
                                {
                                    Name = portName
                                }
                            },
                        }
                    }
                }
            }
        };
}