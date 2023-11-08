using Pulumi;
using Pulumi.Kubernetes.Core.V1;
using Pulumi.Kubernetes.Helm.V3;
using Pulumi.Kubernetes.Types.Inputs.Helm.V3;
using Unilake.Iac.Kubernetes.Helm.Input;

namespace Unilake.Iac.Kubernetes.Helm;

public class Kafka : KubernetesComponentResource
{
    [Output("name")] 
    public Output<string> Name { get; private set; }

    public Service @Service { get; private set; }
    
    public Kafka(KubernetesEnvironmentContext ctx, Namespace? @namespace = null, KafkaInputArgs? inputArgs = null, 
        string name = "kafka", ComponentResourceOptions? options = null, bool checkNamingConvention = true) 
        : base("pkg:kubernetes:helm:kafka", name, options, checkNamingConvention)
    {
        // Check input
        if (inputArgs == null) throw new ArgumentNullException(nameof(inputArgs));

        // Set default options
        var resourceOptions = CreateOptions(options);
        resourceOptions.Parent = this;
        resourceOptions.Provider = ctx.Provider;

        // Set namespace
        @namespace = SetNamespace(resourceOptions, name, @namespace);
        
        // TODO: add configmap KAFKA_CONFLUENT_SCHEMA_REGISTRY_URL
        
        // Get Kafka args
        var releaseArgs = new ReleaseArgs
        {
            Name = name,
            Chart = "kafka",
            Version = inputArgs.Version,
            Namespace = @namespace.Metadata.Apply(x => x.Name),
            RepositoryOpts = new RepositoryOptsArgs
            {
                Repo = "https://charts.bitnami.com/bitnami"
            },
            Values =
                new
                    InputMap<object> // https://github.com/bitnami/charts/blob/main/bitnami/kafka/values.yaml
                    {
                        {
                            "image", new InputMap<string>
                            {
                                {"tag", inputArgs.ImageTag}
                            }
                        },
                        {
                            "extraEnvVars", new []
                            {
                                new InputMap<object>
                                {
                                    // TODO: kafka karapace schema registry not confirmed to be working when seeing the kafka-ui
                                    {"name", "KAFKA_CFG_KAFKA_CONFLUENT_SCHEMA_REGISTRY_URL"},
                                    {"value", inputArgs.SchemaRegistryUrl}
                                }
                            }
                        }
                    },
            // By default Release resource will wait till all created resources
            // are available. Set this to true to skip waiting on resources being
            // available.
            SkipAwait = false
        };
        
        // Check if a private registry is used
        if(inputArgs.UsePrivateRegsitry)
        {
            var registrySecret = CreateRegistrySecret(ctx, resourceOptions, @namespace.Metadata.Apply(x => x.Name));
            string PrivateRegistryBase = !string.IsNullOrWhiteSpace(inputArgs.PrivateRegistryBase) ? inputArgs.PrivateRegistryBase + "/" : "";
            releaseArgs.Values.Add("global.imageRegistry", PrivateRegistryBase);
            releaseArgs.Values.Add("global.imagePullSecrets", new [] {registrySecret.Metadata.Apply(x => x.Name)});
        }

        // Kafka instance
        var kafkaInstance = new Release(name, releaseArgs, resourceOptions);
        
        // Get output
        var status = kafkaInstance.Status;
        Service = Service.Get(name, Output.All(status).Apply(s => $"{s[0].Namespace}/{s[0].Name}"), resourceOptions);
        Name = kafkaInstance.Name;
    }
}