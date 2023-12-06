using Pulumi;
using Pulumi.Kubernetes.Core.V1;
using Pulumi.Kubernetes.Types.Inputs.Core.V1;
using Pulumi.Kubernetes.Types.Inputs.Meta.V1;

namespace Unilake.Iac.Kubernetes.Resource;

public class KubeNamespace : KubernetesComponentResource
{
    [Output("name")]
    public Output<string> Name { get; private set; }

    /// <summary>
    /// Namespace created by this resource
    /// </summary>
    public Namespace Namespace { get; private set; }

    public KubeNamespace(EnvironmentContext ctx, string name, string appName, ComponentResourceOptions? options = null, bool checkNamingConvention = true)
        : base("unilake:kubernetes:resource:kubenamespace", name, options, checkNamingConvention)
    {
        // set default options
        var resourceOptions = CreateOptions(options);
        resourceOptions.Parent = this;

        // Set args
        var args = new NamespaceArgs
        {
            Metadata = new ObjectMetaArgs
            {
                Name = NamingConvention.GetName(name, ctx),
                Labels = GetLabels(ctx, appName, null, "namespace", "NA"),
            },
        };

        // Create resource
        var ns = new Namespace(name, args, resourceOptions);

        // Set output
        Name = ns.Metadata.Apply(x => x.Name);
        Namespace = ns;
    }

    public KubeNamespace(EnvironmentContext ctx, string name, ComponentResourceOptions? options = null, bool checkNamingConvention = true)
        : base("pkg:kubernetes:resource:kubenamespace", name, options, checkNamingConvention)
    {
        // set default options
        var resourceOptions = CreateOptions(options);
        resourceOptions.Parent = this;

        // Set args
        var args = new NamespaceArgs
        {
            Metadata = new ObjectMetaArgs
            {
                Name = name
            },
        };

        // Create resource
        var ns = new Namespace(name, args, resourceOptions);

        // Set output
        Name = ns.Metadata.Apply(x => x.Name);
        Namespace = ns;
    }


    public static Namespace GetNamespace(string name, KubernetesEnvironmentContext ctx) =>
        Namespace.Get(name, name, new CustomResourceOptions
        {
            Provider = ctx.Provider
        });
}