using Pulumi;
using Pulumi.Kubernetes.Core.V1;
using Pulumi.Kubernetes.Helm.V3;
using Pulumi.Kubernetes.Types.Inputs.Helm.V3;
using Unilake.Iac.Kubernetes.Helm.Input;

namespace Unilake.Iac.Kubernetes.Helm;

/// <summary>
/// https://github.com/haproxytech/helm-charts/tree/main/kubernetes-ingress
/// </summary>
public class HaproxyIngress : KubernetesComponentResource
{
    [Output("name")] 
    public Output<string> Name { get; private set; }

    public Service @Service { get; private set; }
    
    public HaproxyIngress(KubernetesEnvironmentContext ctx, HaproxyIngressArgs? inputArgs = null, Namespace? @namespace = null, 
        string name = "haproxingress", ComponentResourceOptions? options = null, bool checkNamingConvention = true)
        : base("pkg:kubernetes:helm:haproxingress", name, options, checkNamingConvention)
    {
        // Check input
        inputArgs ??= new HaproxyIngressArgs();

        // Set default options
        var resourceOptions = CreateOptions(options);
        resourceOptions.Parent = this;
        resourceOptions.Provider = ctx.Provider;

        // Set namespace
        @namespace = SetNamespace(resourceOptions, name, @namespace);
        
        //Get Haproxy chart and add
        var releaseArgs = new ReleaseArgs
        {
            Name = name,
            Chart = "kubernetes-ingress",
            Version = inputArgs.Version,
            Namespace = @namespace.Metadata.Apply(x => x.Name),
            RepositoryOpts = new RepositoryOptsArgs
            {
                Repo = "https://haproxytech.github.io/helm-charts"
            },
            Values = new InputMap<object> // https://github.com/haproxytech/helm-charts/blob/main/kubernetes-ingress/values.yaml
            {
                ["controller"] = new Dictionary<string, object>
                {
                    ["replicaCount"] = inputArgs.ReplicaCount,
                    ["nodeSelector"] = inputArgs.NodeSelector,
                    ["ingressClassResource"] = new Dictionary<string, object>
                    {
                        ["name"] = inputArgs.IngressClassName
                    },
                    ["ingressClass"] = inputArgs.IngressClassName,
                    ["service"] = new Dictionary<string, object>
                    {
                        ["enabled"] = inputArgs.EnableService,
                        ["type"] = inputArgs.ServiceType,
                        ["labels"] = inputArgs.ServiceLabels,
                        ["externalTrafficPolicy"] = inputArgs.ExternalTrafficPolicy
                    },
                }
            },
            // By default Release resource will wait till all created resources
            // are available. Set this to true to skip waiting on resources being
            // available.
            SkipAwait = false,
        };

        // Check if we need to specify the nodeports
        if (inputArgs.NodePorts.Keys.Count > 0)
        {
            if (!inputArgs.NodePorts.Keys.Contains("http") ||
                !inputArgs.NodePorts.Keys.Contains("https") ||
                !inputArgs.NodePorts.Keys.Contains("stat"))
                throw new ArgumentException("Not all of the expected keys (http, https, stats) are provided");

            releaseArgs.Values.Apply(x =>
            {
                var service = (x["controller"] as Dictionary<string, object>)!["service"] as Dictionary<string, object>;
                service!["nodePorts"] = new Dictionary<string, object>
                {
                    ["http"] = inputArgs.NodePorts["http"],
                    ["https"] = inputArgs.NodePorts["https"],
                    ["stat"] = inputArgs.NodePorts["stat"],
                };
                return x;
            });

            releaseArgs.Values.Apply(x => x.Add("controller.service.nodePorts.http", inputArgs.NodePorts["http"]));
            releaseArgs.Values.Apply(x => x.Add("controller.service.nodePorts.https", inputArgs.NodePorts["https"]));
            releaseArgs.Values.Apply(x => x.Add("controller.service.nodePorts.stat", inputArgs.NodePorts["stat"]));
        }
        
        // Check if servicemonitor should be enabled
        if (inputArgs.EnableServiceMonitor)
            releaseArgs.Values.Apply(x => x.Add("controller.serviceMonitor.enabled", true));

        // Check if a private registry is used
        if(inputArgs.UsePrivateRegsitry)
            throw new NotImplementedException("Private registry currently not supported");

        // HaproxyIngress instance
        var haproxyInstance = new Release(name, releaseArgs, resourceOptions);
        
        // Get output
        var status = haproxyInstance.Status;
        @Service = Service.Get(name, Output.All(status).Apply(s => $"{s[0].Namespace}/{s[0].Name}-kubernetes-ingress"), resourceOptions);
        Name = haproxyInstance.Name;
    }
}