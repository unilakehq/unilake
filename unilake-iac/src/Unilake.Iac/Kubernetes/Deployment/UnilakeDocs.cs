using Pulumi;
using Pulumi.Kubernetes.Core.V1;
using Pulumi.Kubernetes.Types.Inputs.Apps.V1;
using Pulumi.Kubernetes.Types.Inputs.Core.V1;
using Pulumi.Kubernetes.Types.Inputs.Meta.V1;
using Unilake.Iac.Kubernetes.Deployment.Input;
using Unilake.Iac.Kubernetes.Resource;

namespace Unilake.Iac.Kubernetes.Deployment;

/// <summary>
/// Unilake public documentation site
/// </summary>
public sealed class UnilakeDocs : KubernetesComponentResource
{
    public UnilakeDocs(KubernetesEnvironmentContext ctx, UnilakeDocsInputArgs inputArgs, Namespace? @namespace = null,
        string name = "unilake-docs", ComponentResourceOptions? options = null)
        : base("unilake:kubernetes:deployment:unilake:docs", name, options)
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
                    { "app", "unilake-docs" },
                },
            },
            Spec = new DeploymentSpecArgs
            {
                Replicas = 1,
                Selector = new LabelSelectorArgs
                {
                    MatchLabels =
                    {
                        { "app", "unilake-docs" },
                    },
                },
                Template = new PodTemplateSpecArgs
                {
                    Metadata = new ObjectMetaArgs
                    {
                        Labels =
                        {
                            { "app", "unilake-docs" },
                        },
                    },
                    Spec = new PodSpecArgs
                    {
                        Containers =
                        {
                            new ContainerArgs
                            {
                                Name = "unilake-docs",
                                Image = $"ghcr.io/unilakehq/docs:{inputArgs.Version}",
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
                    { "app", "unilake-docs" },
                },
            }
        };

        // create service
        // sonarlint-disable csharpsquid:S1481
        // sonarlint-disable S1482
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
