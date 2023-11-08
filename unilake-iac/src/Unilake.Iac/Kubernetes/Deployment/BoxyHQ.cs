using Pulumi;
using Pulumi.Kubernetes.Core.V1;
using Pulumi.Kubernetes.Types.Inputs.Apps.V1;
using Pulumi.Kubernetes.Types.Inputs.Core.V1;
using Pulumi.Kubernetes.Types.Inputs.Meta.V1;
using Unilake.Iac.Kubernetes.Deployment.Input;

namespace Unilake.Iac.Kubernetes.Deployment;

/// <summary>
/// TODO: some settings are missing to make everything work properly
/// </summary>
public class BoxyHQ : KubernetesDeploymentResource
{
    public BoxyHQ(KubernetesEnvironmentContext ctx, Namespace? @namespace = null, BoxyHqInputArgs? inputArgs = null,
        string name = "boxyhq", ComponentResourceOptions? options = null, bool checkNamingConvention = true)
        : base("pkg:kubernetes:deployment:boxyhq", name, options, checkNamingConvention)
    {
        // See: https://boxyhq.com/docs/jackson/deploy/service

        // set default options
        var resourceOptions = CreateOptions(options);
        resourceOptions.Parent = this;
        resourceOptions.Provider = ctx.Provider;

        // set resource
        string version = inputArgs?.Version ?? "latest";
        var labels = GetLabels(ctx, name, inputArgs?.PartOf, "boxyhq", version);

        // Set namespace
        @namespace = SetNamespace(resourceOptions, name, @namespace);

        // create secret
        CreateSecret(@namespace, name, GetLabels(ctx, name, inputArgs?.PartOf, "boxyhq", version), new InputMap<string>
        {
            { "JACKSON_API_KEYS", inputArgs.JacksonApiKey.Apply(x => string.Join(",", x).EncodeBase64()) },
            {
                "DB_URL", Output.All(
                        inputArgs.DbUsername.ToOutput(),
                        inputArgs.DbPassword.ToOutput(),
                        inputArgs.DbEndpoint.ToOutput(),
                        inputArgs.DbPort.Apply(x => x.ToString()),
                        inputArgs.DbDatabaseName.ToOutput<string>())
                    .Apply(x =>
                        // postgres://admin:unknown@postgres:5432/app
                        $"{inputArgs.DbType}://{x[0]}:{x[1]}@{x[2]}:{x[3]}/{x[4]}".EncodeBase64()
                    )
            }
        }, resourceOptions);

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
                    { "app", "boxyhq" },
                },
            },
            Spec = new DeploymentSpecArgs
            {
                Replicas = 2,
                Strategy = new DeploymentStrategyArgs
                {
                    Type = "RollingUpdate",
                    RollingUpdate = new RollingUpdateDeploymentArgs
                    {
                        MaxUnavailable = 1,
                        MaxSurge = 1,
                    },
                },
                Selector = new LabelSelectorArgs
                {
                    MatchLabels =
                    {
                        { "app", "boxyhq" },
                    },
                },
                Template = new PodTemplateSpecArgs
                {
                    Metadata = new ObjectMetaArgs
                    {
                        Labels =
                        {
                            { "app", "boxyhq" },
                        },
                    },
                    Spec = new PodSpecArgs
                    {
                        // ImagePullSecrets = 
                        // {
                        //     new LocalObjectReferenceArgs
                        //     {
                        //         Name = "docker-secret",
                        //     },
                        // },
                        Containers =
                        {
                            new ContainerArgs
                            {
                                Name = "boxyhq",
                                Image = $"boxyhq/jackson:{version}",
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
                                        ContainerPortValue = 5225,
                                    },
                                },
                                ReadinessProbe = new ProbeArgs
                                {
                                    HttpGet = new HTTPGetActionArgs
                                    {
                                        Port = 5225,
                                        Path = "/api/health",
                                    },
                                    SuccessThreshold = 1,
                                    InitialDelaySeconds = 10,
                                    PeriodSeconds = 5,
                                    FailureThreshold = 6,
                                },
                                Env = // For more information: https://boxyhq.com/docs/jackson/deploy/env-variables
                                {
                                    new EnvVarArgs
                                    {
                                        Name = "JACKSON_API_KEYS",
                                        ValueFrom = new EnvVarSourceArgs
                                        {
                                            SecretKeyRef = new SecretKeySelectorArgs
                                            {
                                                Name = name,
                                                Key = "JACKSON_API_KEYS",
                                            },
                                        },
                                    },
                                    new EnvVarArgs
                                    {
                                        Name = "DB_ENGINE",
                                        Value = inputArgs.DbEngine
                                    },
                                    new EnvVarArgs
                                    {
                                        Name = "DB_TYPE",
                                        Value = inputArgs.DbType
                                    },
                                    new EnvVarArgs
                                    {
                                        Name = "DB_URL",
                                        ValueFrom = new EnvVarSourceArgs
                                        {
                                            SecretKeyRef = new SecretKeySelectorArgs
                                            {
                                                Name = name,
                                                Key = "DB_URL",
                                            },
                                        },
                                    },
                                },
                            },
                        },
                    },
                },
            },
        };

        // Create resource
        var boxyhqDeployment = new global::Pulumi.Kubernetes.Apps.V1.Deployment(name, args, resourceOptions);

        // Create service
        var serviceargs = new ServiceArgs
        {
            Metadata = new ObjectMetaArgs
            {
                Name = name,
                Namespace = @namespace.Metadata.Apply(x => x.Name)
                //Labels = labels
            },
            Spec = new ServiceSpecArgs
            {
                Ports =
                {
                    new ServicePortArgs
                    {
                        Name = "http",
                        Port = 5225,
                    }
                },
                Selector =
                {
                    { "app", "boxyhq" },
                },
            }
        };

        // Create service
        var service = new Service(name, serviceargs, resourceOptions);
    }
}