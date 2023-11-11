using Pulumi;
using Pulumi.Kubernetes.Core.V1;
using Pulumi.Kubernetes.Helm.V3;
using Pulumi.Kubernetes.Types.Inputs.Helm.V3;
using Unilake.Iac.Kubernetes.Helm.Input;

namespace Unilake.Iac.Kubernetes.Helm;

public class KafkaUi : KubernetesComponentResource
{
    [Output("name")] 
    public Output<string> Name { get; private set; }

    public Service @Service { get; private set; }
    
    public KafkaUi(KubernetesEnvironmentContext ctx, KafkaUiInputArgs inputArgs, Namespace? @namespace = null, string name = "kafkaui", 
        ComponentResourceOptions? options = null, bool checkNamingConvention = true) 
        : base("pkg:kubernetes:helm:kafkaui", name, options, checkNamingConvention)
    {
        // Set default options
        var resourceOptions = CreateOptions(options);
        resourceOptions.Parent = this;
        resourceOptions.Provider = ctx.Provider;

        // Set namespace
        @namespace = SetNamespace(resourceOptions, name, @namespace);
        
        // Get Kafka args
        var releaseArgs = new ReleaseArgs
        {
            Name = name,
            Chart = "kafka-ui",
            Version = inputArgs.Version,
            Namespace = @namespace.Metadata.Apply(x => x.Name),
            RepositoryOpts = new RepositoryOptsArgs
            {
                Repo = "https://provectus.github.io/kafka-ui"
            },
            Values =
                new
                    InputMap<object> // https://github.com/provectus/kafka-ui/blob/master/charts/kafka-ui/values.yaml
                    {
                        {
                            "yamlApplicationConfig", new InputMap<object>
                            {
                                {
                                    "kafka", new InputMap<object>
                                    {
                                        {
                                            "clusters", new[]
                                            {
                                                new InputMap<object>
                                                {
                                                    {"name", inputArgs.ServerName},
                                                    {"bootstrapServers", inputArgs.ServerBootstrapAddress}
                                                }
                                            }
                                        }
                                    }
                                },
                                {
                                    "auth", new InputMap<object>
                                    {
                                        {"type", "disabled"}
                                    }
                                },
                                {
                                    "management", new InputMap<object>
                                    {
                                        {
                                            "health", new InputMap<object>
                                            {
                                                {
                                                    "ldap", new InputMap<object>
                                                    {
                                                        {"enabled", "false"}
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    },
            // By default Release resource will wait till all created resources
            // are available. Set this to true to skip waiting on resources being
            // available.
            SkipAwait = false
        };
        
        // Kafka-ui instance
        var kafkaUiInstance = new Release(name, releaseArgs, resourceOptions);
        
        // Get output
        var status = kafkaUiInstance.Status;
        Service = Service.Get(name, Output.All(status).Apply(s => $"{s[0].Namespace}/{s[0].Name}"), resourceOptions);
        Name = kafkaUiInstance.Name; 
    }
}