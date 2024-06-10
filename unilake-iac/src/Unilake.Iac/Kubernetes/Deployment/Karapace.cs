using Pulumi;
using Pulumi.Kubernetes.Core.V1;
using Pulumi.Kubernetes.Types.Inputs.Apps.V1;
using Pulumi.Kubernetes.Types.Inputs.Core.V1;
using Pulumi.Kubernetes.Types.Inputs.Meta.V1;
using Unilake.Iac.Kubernetes.Deployment.Input;

namespace Unilake.Iac.Kubernetes.Deployment;

public sealed class Karapace : KubernetesComponentResource
{
    public const string RegistryName = "karapace-registry";
    public const string RestName = "karapace-rest";
    private string _imageVersion;

    public Service RegistryService { get; private set; }
    public Service RestService { get; private set; }

    public Karapace(KubernetesEnvironmentContext ctx, KarapaceInputArgs? inputArgs = null, Namespace? @namespace = null, string name = "karapace",
        ComponentResourceOptions? options = null, bool checkNamingConvention = true)
        : base("unilake:kubernetes:deployment:karapace", name, options, checkNamingConvention)
    {
        // TODO: https://github.com/aiven/karapace/blob/main/container/compose.yml
        // Step 1: translate to yaml https://kubernetes.io/docs/tasks/configure-pod-container/translate-compose-kubernetes/
        // Step 2: translate to pulumi https://www.pulumi.com/kube2pulumi/

        // Also check: https://github.com/aiven/karapace/pull/257

        // Check input
        if (inputArgs == null)
            throw new ArgumentNullException(nameof(inputArgs), "InputArgs cannot be null");
        if (string.IsNullOrWhiteSpace(inputArgs.ImageVersion))
            throw new ArgumentNullException(nameof(inputArgs.ImageVersion), "ImageVersion cannot be null");
        _imageVersion = inputArgs.ImageVersion;

        // Set default options
        var resourceOptions = CreateOptions(options);
        resourceOptions.Parent = this;
        resourceOptions.Provider = ctx.Provider;

        // Set namespace
        @namespace = SetNamespace(resourceOptions, name, @namespace);

        // set resources
        var registryPod = GetKarapaceRegistryPod(@namespace, inputArgs.KafkaServiceName, inputArgs.KafkaServicePort, resourceOptions);
        var restPod = GetKarapaceRestPod(@namespace, inputArgs.KafkaServiceName, inputArgs.KafkaServicePort, resourceOptions);
        RegistryService = GetRegistryService(@namespace, registryPod, resourceOptions);
        RestService = GetRestService(@namespace, restPod, resourceOptions);
    }

    Pulumi.Kubernetes.Apps.V1.Deployment GetKarapaceRegistryPod(Namespace @namespace, string kafkaservicename, int kafkaserviceport,
        CustomResourceOptions resourceOptions, string name = RegistryName) => new(name, new DeploymentArgs
        {
            ApiVersion = "apps/v1",
            Kind = "Deployment",
            Metadata = new ObjectMetaArgs
            {
                Name = name,
                Namespace = @namespace.Metadata.Apply(x => x.Name),
            },
            Spec = new DeploymentSpecArgs
            {
                Replicas = 1,
                Selector = new LabelSelectorArgs
                {
                    MatchLabels =
                    {
                        { "app.kubernetes.io/name", name },
                    },
                },
                Template = new PodTemplateSpecArgs
                {
                    Metadata = new ObjectMetaArgs
                    {
                        Labels =
                        {
                            { "app.kubernetes.io/name", name },
                        },
                    },
                    Spec = new PodSpecArgs
                    {
                        Containers =
                        {
                            new ContainerArgs
                            {
                                Name = name,
                                Image = $"ghcr.io/aiven-open/karapace:{_imageVersion}",
                                Ports =
                                {
                                    new ContainerPortArgs
                                    {
                                        Name = "http",
                                        ContainerPortValue = 8081,
                                    },
                                },
                                Command = new List<string>
                                {
                                    "/bin/bash",
                                    "/opt/karapace/start.sh",
                                    "registry"
                                },
                                Env = new InputList<EnvVarArgs>
                                {
                                    new EnvVarArgs
                                    {
                                        Name = "KARAPACE_ADVERTISED_HOSTNAME",
                                        Value = name
                                    },
                                    new EnvVarArgs
                                    {
                                        Name = "KARAPACE_BOOTSTRAP_URI",
                                        Value = $"{kafkaservicename}:{kafkaserviceport}"
                                    },
                                    new EnvVarArgs
                                    {
                                        Name = "KARAPACE_PORT",
                                        Value = "8081"
                                    },
                                    new EnvVarArgs
                                    {
                                        Name = "KARAPACE_HOST",
                                        Value = "0.0.0.0"
                                    },
                                    new EnvVarArgs
                                    {
                                        Name = "KARAPACE_CLIENT_ID",
                                        Value = "karapace"
                                    },
                                    new EnvVarArgs
                                    {
                                        Name = "KARAPACE_GROUP_ID",
                                        Value = "karapace-registry"
                                    },
                                    new EnvVarArgs
                                    {
                                        Name = "KARAPACE_MASTER_ELIGIBILITY",
                                        Value = "true"
                                    },
                                    new EnvVarArgs
                                    {
                                        Name = "KARAPACE_TOPIC_NAME",
                                        Value = "_schemas"
                                    },
                                    new EnvVarArgs
                                    {
                                        Name = "KARAPACE_LOG_LEVEL",
                                        Value = "WARNING"
                                    },
                                    new EnvVarArgs
                                    {
                                        Name = "KARAPACE_COMPATIBILITY",
                                        Value = "FULL"
                                    },
                                }
                            },
                        },
                    },
                },
            },
        }, resourceOptions);

    Pulumi.Kubernetes.Apps.V1.Deployment GetKarapaceRestPod(Namespace @namespace, string kafkaservicename, int kafkaserviceport,
        CustomResourceOptions resourceOptions, string registryName = RegistryName, string name = RestName) => new(name, new DeploymentArgs
        {
            ApiVersion = "apps/v1",
            Kind = "Deployment",
            Metadata = new ObjectMetaArgs
            {
                Name = name,
                Namespace = @namespace.Metadata.Apply(x => x.Name),
            },
            Spec = new DeploymentSpecArgs
            {
                Replicas = 1,
                Selector = new LabelSelectorArgs
                {
                    MatchLabels =
                    {
                        { "app.kubernetes.io/name", name },
                    },
                },
                Template = new PodTemplateSpecArgs
                {
                    Metadata = new ObjectMetaArgs
                    {
                        Labels =
                        {
                            { "app.kubernetes.io/name", name },
                        },
                    },
                    Spec = new PodSpecArgs
                    {
                        Containers =
                        {
                            new ContainerArgs
                            {
                                Name = name,
                                Image = $"ghcr.io/aiven-open/karapace:{_imageVersion}",
                                Ports =
                                {
                                    new ContainerPortArgs
                                    {
                                        Name = "http",
                                        ContainerPortValue = 8082,
                                    },
                                },
                                Command = new List<string>
                                {
                                    "/bin/bash",
                                    "/opt/karapace/start.sh",
                                    "rest"
                                },
                                Env = new InputList<EnvVarArgs>
                                {
                                    new EnvVarArgs
                                    {
                                        Name = "KARAPACE_ADVERTISED_HOSTNAME",
                                        Value = name
                                    },
                                    new EnvVarArgs
                                    {
                                        Name = "KARAPACE_BOOTSTRAP_URI",
                                        Value = $"{kafkaservicename}:{kafkaserviceport}"
                                    },
                                    new EnvVarArgs
                                    {
                                        Name = "KARAPACE_PORT",
                                        Value = "8082"
                                    },
                                    new EnvVarArgs
                                    {
                                        Name = "KARAPACE_HOST",
                                        Value = "0.0.0.0"
                                    },
                                    new EnvVarArgs
                                    {
                                        Name = "KARAPACE_REGISTRY_HOST",
                                        Value = registryName
                                    },
                                    new EnvVarArgs
                                    {
                                        Name = "KARAPACE_REGISTRY_PORT",
                                        Value = "8081"
                                    },
                                    new EnvVarArgs
                                    {
                                        Name = "KARAPACE_ADMIN_METADATA_MAX_AGE",
                                        Value = "0"
                                    },
                                }
                            },
                        },
                    },
                },
            },
        }, resourceOptions);

    Service GetRegistryService(Namespace @namespace, Pulumi.Kubernetes.Apps.V1.Deployment registryPod, CustomResourceOptions resourceOptions, string name = RegistryName + "-service") => new(name, new ServiceArgs
    {
        ApiVersion = "v1",
        Kind = "Service",
        Metadata = new ObjectMetaArgs
        {
            Name = name,
            Namespace = @namespace.Metadata.Apply(x => x.Name),
            Labels =
            {
                { "app.kubernetes.io/name", name },
            },
        },
        Spec = new ServiceSpecArgs
        {
            Type = "ClusterIP",
            Ports =
            {
                new ServicePortArgs
                {
                    Port = 8081,
                    TargetPort = "http",
                    Name = name,
                },
            },
            Selector =
            {
                { "app.kubernetes.io/name", registryPod.Metadata.Apply(x => x.Name) },
            },
        },
    }, resourceOptions);

    Service GetRestService(Namespace @namespace, Pulumi.Kubernetes.Apps.V1.Deployment restPod, CustomResourceOptions resourceOptions, string name = RestName + "-service") => new(name, new ServiceArgs
    {
        ApiVersion = "v1",
        Kind = "Service",
        Metadata = new ObjectMetaArgs
        {
            Name = name,
            Namespace = @namespace.Metadata.Apply(x => x.Name),
            Labels =
            {
                { "app.kubernetes.io/name", name },
            },
        },
        Spec = new ServiceSpecArgs
        {
            Type = "ClusterIP",
            Ports =
            {
                new ServicePortArgs
                {
                    Port = 8082,
                    TargetPort = "http",
                    Name = name,
                },
            },
            Selector =
            {
                { "app.kubernetes.io/name", restPod.Metadata.Apply(x => x.Name) },
            },
        },
    }, resourceOptions);
}