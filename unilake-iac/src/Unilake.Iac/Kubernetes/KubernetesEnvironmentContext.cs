using Pulumi;
using Pulumi.Kubernetes;
using Pulumi.Kubernetes.Core.V1;

namespace Unilake.Iac.Kubernetes;

/// <summary>
/// TODO: improve the factory methods, currently the first one is used instead of the second one
/// </summary>
public class KubernetesEnvironmentContext : EnvironmentContext
{
    public Provider Provider { get; protected set; }

    /// <summary>
    /// If needed, set a default namespace where resources should deployed into
    /// </summary>
    public string DefaultNamespace { get; set; } = string.Empty;

    private KubernetesEnvironmentContext(EnvironmentContext ctx) : base(ctx)
    {
        
    }
    
    /// <summary>
    /// Create a new KubernetesEnvironmentContext that makes use of the default kubeconfig
    /// </summary>
    /// <param name="ctx"></param>
    /// <param name="name"></param>
    /// <param name="renderToYamlDirectory"></param>
    /// <returns></returns>
    public static KubernetesEnvironmentContext Create(EnvironmentContext ctx, string name, Input<string>? renderToYamlDirectory = null)
    {
        var k8s = new KubernetesEnvironmentContext(ctx.Copy());

        k8s.Provider = new Provider(name, new ProviderArgs
        {
            RenderYamlToDirectory = renderToYamlDirectory,
            EnableServerSideApply = true
        });

        return k8s;
    }
    
    /// <summary>
    /// Create a new KubernetesEnvironmentContext that makes use of a specified kubeconfig
    /// </summary>
    /// <param name="ctx"></param>
    /// <param name="name"></param>
    /// <param name="kubeconfig"></param>
    /// <param name="renderToYamlDirectory"></param>
    /// <returns></returns>
    public static KubernetesEnvironmentContext Create(EnvironmentContext ctx, string name, Input<string> kubeconfig, Input<string>? renderToYamlDirectory = null)
    {
        var k8s = new KubernetesEnvironmentContext(ctx.Copy());

        k8s.Provider = new Provider(name, new ProviderArgs
        {
            KubeConfig = kubeconfig,
            RenderYamlToDirectory = renderToYamlDirectory,
            EnableServerSideApply = true
        });

        return k8s;
    }

    /// <summary>
    /// Get a config value from either an environment variable or pulumi config value
    /// </summary>
    public Output<string> GetConfigValue(string name) => Output.Create(System.Environment.GetEnvironmentVariable(name)
        ?? Config.Get(name)
        ?? throw new ArgumentException($"Cannot find config value {name}"));

    /// <summary>
    /// Get a config value from either an environment variable or pulumi config value as secret
    /// </summary>
    public Output<string> GetConfigSecret(string name) => Output.CreateSecret(System.Environment.GetEnvironmentVariable(name))
        ?? Config.GetSecret(name)
        ?? throw new ArgumentException($"Cannot find config secret {name}");


    /// <summary>
    /// Get an existing namespace
    /// </summary>
    public Namespace GetNamespace(string name) => Namespace.Get(name, name, new CustomResourceOptions
    {
        Provider = this.Provider
    });
}