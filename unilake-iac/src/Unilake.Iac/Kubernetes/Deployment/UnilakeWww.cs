using Pulumi;
using Pulumi.Kubernetes.Core.V1;
using Pulumi.Kubernetes.Types.Inputs.Apps.V1;
using Pulumi.Kubernetes.Types.Inputs.Core.V1;
using Pulumi.Kubernetes.Types.Inputs.Meta.V1;
using Unilake.Iac.Kubernetes.Resource;

namespace Unilake.Iac.Kubernetes.Deployment;

/// <summary>
/// Unilake public website
/// </summary>
public sealed class UnilakeWww : KubernetesComponentResource
{
    public UnilakeWww(KubernetesEnvironmentContext ctx, UnilakeWwwInputArgs inputArgs, Namespace? @namespace = null,
        string name = "unilake-www", ComponentResourceOptions? options = null)
        : base("unilake:kubernetes:deployment:unilake:www", name, options)
    {
        // check input
        if (inputArgs == null) throw new ArgumentNullException(nameof(inputArgs));

        // set default options
        var resourceOptions = CreateOptions(options);
        resourceOptions.Parent = this;
        resourceOptions.Provider = ctx.Provider;

        // Set namespace
        @namespace = SetNamespace(resourceOptions, name, @namespace);

        // create deployment
        var args = new DeploymentArgs
        {
            ApiVersion = "apps/v1",
            Kind = "Deployment",
            Metadata = new ObjectMetaArgs
            {
                Name = name,
                Namespace = @namespace.Metadata.Apply(x => x.Name),
                Labels =
                {
                    { "app", "unilake-www" },
                },
            },
            Spec = new DeploymentSpecArgs
            {
                Replicas = 1,
                Selector = new LabelSelectorArgs
                {
                    MatchLabels =
                    {
                        { "app", "unilake-www" },
                    },
                },
                Template = new PodTemplateSpecArgs
                {
                    Metadata = new ObjectMetaArgs
                    {
                        Labels =
                        {
                            { "app", "unilake-www" },
                        },
                    },
                    Spec = new PodSpecArgs
                    {
                        Containers =
                        {
                            new ContainerArgs
                            {
                                Name = "unilake-www",
                                Image = $"ghcr.io/unilakehq/www:{inputArgs.Version}",
                                ImagePullPolicy = "Always",
                                Resources = new ResourceRequirementsArgs
                                {
                                    Limits =
                                    {
                                        { "memory", "2000Mi" },
                                        { "cpu", "2000m" },
                                    },
                                    Requests =
                                    {
                                        { "memory", "1000Mi" },
                                        { "cpu", "1000m" },
                                    },
                                },
                                Ports =
                                {
                                    new ContainerPortArgs
                                    {
                                        ContainerPortValue = 80,
                                        Name = "http",
                                        Protocol = "TCP",
                                    },
                                },
                                ReadinessProbe = new ProbeArgs
                                {
                                    HttpGet = new HTTPGetActionArgs
                                    {
                                        Port = 80,
                                        Path = "/",
                                    },
                                    SuccessThreshold = 1,
                                    InitialDelaySeconds = 10,
                                    PeriodSeconds = 5,
                                    FailureThreshold = 6,
                                },
                            },
                        },
                    },
                },
            },
        };

        // create resource
        var deployment = new Pulumi.Kubernetes.Apps.V1.Deployment(name, args, resourceOptions);

        // create service
        var serviceargs = new ServiceArgs
        {
            Metadata = new ObjectMetaArgs
            {
                Name = name,
                Namespace = @namespace.Metadata.Apply(x => x.Name)
            },
            Spec = new ServiceSpecArgs
            {
                Ports =
                {
                    new ServicePortArgs
                    {
                        Name = "http",
                        Port = 80,
                    }
                },
                Selector =
                {
                    { "app", "unilake-www" },
                },
            }
        };

        // create service
        var service = new Service(name, serviceargs, resourceOptions);

        // create ingress
        if (string.IsNullOrWhiteSpace(inputArgs.Url))
            return;

        var ingress = new Ingress(ctx, name, new Pulumi.Kubernetes.Types.Inputs.Networking.V1.IngressSpecArgs
        {
            Rules = Ingress.CreateServiceIngressRule(inputArgs.Url, "/", "Prefix", name, "http")
        }, @namespace);
    }
}