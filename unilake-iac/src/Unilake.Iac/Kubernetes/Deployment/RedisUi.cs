using System.Text.Json;
using Pulumi;
using Pulumi.Kubernetes.Core.V1;
using Pulumi.Kubernetes.Types.Inputs.Apps.V1;
using Pulumi.Kubernetes.Types.Inputs.Core.V1;
using Pulumi.Kubernetes.Types.Inputs.Meta.V1;
using Unilake.Iac.Kubernetes.Deployment.Input;
using Unilake.Iac.Kubernetes.Helm;

namespace Unilake.Iac.Kubernetes.Deployment;

public class RedisUi : KubernetesDeploymentResource
{
    public Service @Service { get; private set; }

    public RedisUi(KubernetesEnvironmentContext ctx, Redis instance, Namespace? @namespace = null,
        string name = "redis-ui", RedisUiArgs? args = null, ComponentResourceOptions? options = null, bool checkNamingConvention = true)
        : base("pkg:kubernetes:deployment:redis-ui", name, options, checkNamingConvention)
    {
        // Check input
        args ??= new RedisUiArgs();
        
        // Set default options
        var resourceOptions = CreateOptions(options);
        resourceOptions.Parent = this;
        resourceOptions.Provider = ctx.Provider;

        // Set namespace
        @namespace = SetNamespace(resourceOptions, name, @namespace);

        // Create configmap
        var configMap = new ConfigMap("redis-ui-configmap", new ConfigMapArgs
        {
            ApiVersion = "v1",
            Kind = "ConfigMap",
            Metadata = new ObjectMetaArgs
            {
                Name = "p3x-redis-ui-settings",
                Namespace = @namespace.Metadata.Apply(x => x.Name),
            },
            Data =
            {
                {
                    ".p3xrs-conns.json", Output.All(instance.Name, instance.Service.Spec.Apply(x => x.ClusterIP), instance.Password).Apply(x => 
                            JsonSerializer.Serialize(new
                            {
                                list = new object[]
                                {
                                    new
                                    {
                                        name = x[0],
                                        host =  x[1],
                                        port = 6379,
                                        password = x[2],
                                        id = "unique"
                                    }
                                },
                                license = ""
                            })
                        )
                },
            },
        }, resourceOptions);

        // Make sure we are creating the configmap first
        resourceOptions.DependsOn = configMap;

        // Deployment
        var _ = new global::Pulumi.Kubernetes.Apps.V1.Deployment("redis-ui-deployment", new DeploymentArgs
        {
            ApiVersion = "apps/v1",
            Kind = "Deployment",
            Metadata = new ObjectMetaArgs
            {
                Name = "p3x-redis-ui",
                Namespace = @namespace.Metadata.Apply(x => x.Name),
            },
            Spec = new DeploymentSpecArgs
            {
                Replicas = 1,
                Selector = new LabelSelectorArgs
                {
                    MatchLabels =
                    {
                        { "app.kubernetes.io/name", "p3x-redis-ui" },
                    },
                },
                Template = new PodTemplateSpecArgs
                {
                    Metadata = new ObjectMetaArgs
                    {
                        Labels =
                        {
                            { "app.kubernetes.io/name", "p3x-redis-ui" },
                        },
                    },
                    Spec = new PodSpecArgs
                    {
                        Containers =
                        {
                            new ContainerArgs
                            {
                                Name = "p3x-redis-ui",
                                Image = "patrikx3/p3x-redis-ui",
                                Ports =
                                {
                                    new ContainerPortArgs
                                    {
                                        Name = "p3x-redis-ui",
                                        ContainerPortValue = 7843,
                                    },
                                },
                                VolumeMounts =
                                {
                                    new VolumeMountArgs
                                    {
                                        Name = "p3x-redis-ui-settings",
                                        MountPath = "/settings/.p3xrs-conns.json",
                                        SubPath = ".p3xrs-conns.json",
                                    },
                                },
                            },
                        },
                        Volumes =
                        {
                            new VolumeArgs
                            {
                                Name = "p3x-redis-ui-settings",
                                ConfigMap = new ConfigMapVolumeSourceArgs
                                {
                                    Name = "p3x-redis-ui-settings",
                                },
                            },
                        },
                    },
                },
            },
        }, resourceOptions);

        // Service
        var service = new Service("redis-ui-service", new ServiceArgs
        {
            ApiVersion = "v1",
            Kind = "Service",
            Metadata = new ObjectMetaArgs
            {
                Name = "p3x-redis-ui-service",
                Namespace = @namespace.Metadata.Apply(x => x.Name),
                Labels =
                {
                    { "app.kubernetes.io/name", "p3x-redis-ui-service" },
                },
            },
            Spec = new ServiceSpecArgs
            {
                Type = args.ServiceType,
                Ports =
                {
                    new ServicePortArgs
                    {
                        Port = 7843,
                        TargetPort = "p3x-redis-ui",
                        Name = "p3x-redis-ui",
                    },
                },
                Selector =
                {
                    { "app.kubernetes.io/name", "p3x-redis-ui" },
                },
            },
        }, resourceOptions);

        // Set the output service
        Service = service;
    }
}