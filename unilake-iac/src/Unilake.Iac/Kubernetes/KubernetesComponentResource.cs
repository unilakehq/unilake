using System.Text;
using Pulumi;
using Pulumi.Kubernetes.Core.V1;
using Pulumi.Kubernetes.Types.Inputs.Core.V1;
using Pulumi.Kubernetes.Types.Inputs.Meta.V1;

namespace Unilake.Iac.Kubernetes;

/// <summary>
/// Check if all best practices are adhered to: https://learnk8s.io/production-best-practices
/// </summary>
public class KubernetesComponentResource : InternalComponentResource<NamingConventionKubernetesResource>
{
    public KubernetesComponentResource(string type, string name, ComponentResourceOptions? options = null,
        bool checkNamingConvention = true) : base(type,
        name, options, checkNamingConvention)
    {

    }

    public KubernetesComponentResource(string type, string name, ResourceArgs? args,
        ComponentResourceOptions? options = null, bool remote = false) : base(type, name, args, options, remote) =>
        throw new NotImplementedException();

    protected CustomResourceOptions CreateOptions(ComponentResourceOptions? options = null) =>
        new()
        {
            DependsOn = options?.DependsOn ?? new InputList<Resource>(),
            Parent = options?.Parent ?? null,
            Provider = options?.Provider ?? null,
        };

    protected ComponentResourceOptions? CopyOptions(ComponentResourceOptions? options = null) => options == null
        ? options
        : new ComponentResourceOptions
        {
            Aliases = options.Aliases,
            Id = options.Id,
            Parent = options.Parent,
            Protect = options.Protect,
            Provider = options.Provider,
            Providers = options.Providers,
            Urn = options.Urn,
            Version = options.Version,
            CustomTimeouts = options.CustomTimeouts,
            DependsOn = options.DependsOn,
            IgnoreChanges = options.IgnoreChanges,
            ResourceTransformations = options.ResourceTransformations,
            ReplaceOnChanges = options.ReplaceOnChanges,
            RetainOnDelete = options.RetainOnDelete,
            PluginDownloadURL = options.PluginDownloadURL
        };

    /// <summary>
    /// Get the labels associated to this resource
    /// </summary>
    /// <param name="ctx">Deployment Context</param>
    /// <param name="appName">The name of the application</param>
    /// <param name="partOf">The name of a higher level application this one is part of</param>
    /// <param name="component">The component within the architecture</param>
    /// <param name="version">The current version of the application (e.g., a semantic version, revision hash, etc.)</param>
    /// <param name="additionalLabels">Any additional labels to use</param>
    /// <returns></returns>
    public Dictionary<string, string> GetLabels(EnvironmentContext ctx, string appName, string? partOf,
        string component, string version,
        Dictionary<string, string>? additionalLabels = null)
    {
        // check input
        if (string.IsNullOrWhiteSpace(appName))
            throw new ArgumentNullException(nameof(appName));
        if (string.IsNullOrWhiteSpace(component))
            throw new ArgumentNullException(nameof(component));
        if (string.IsNullOrWhiteSpace(version))
            throw new ArgumentNullException(nameof(version));
        if (string.IsNullOrWhiteSpace(partOf))
            partOf = "unknown";

        // Based on https://kubernetes.io/docs/concepts/overview/working-with-objects/common-labels/#labels
        string instance =
            $"{appName}-{ctx.Domain}-{NamingConvention.GetEnvironmentSequence(ctx.EnvironmentSequence)}";
        if (ctx.ResourceSequence > 0)
            instance = $"{instance}-{NamingConvention.GetResourceSequence(ctx.ResourceSequence)}";

        var toreturn = new Dictionary<string, string>
        {
            { "app.kubernetes.io/name", appName },
            { "app.kubernetes.io/managed-by", "pulumi" },
            { "app.kubernetes.io/part-of", partOf },
            { "app.kubernetes.io/component", component },
            { "app.kubernetes.io/instance", instance },
            { "app.kubernetes.io/version", version },
            { "unilake/tenant", ctx.Tenant },
            { "unilake/environment-sequence", NamingConvention.GetEnvironmentSequence(ctx.EnvironmentSequence) },
            { "unilake/domain", ctx.Domain },
            { "unilake/environment", ctx.Environment },
            { "unilake/region", ctx.Region },
            { "unilake/cloud-provider", ctx.CloudProvider },
        };

        if (additionalLabels != null)
            foreach (var item in additionalLabels)
                toreturn.Add(item.Key, item.Value);

        foreach (var item in ctx.CustomTags)
            toreturn.Add(item.Key, item.Value);

        if (ctx.ResourceSequence > 0)
            toreturn.Add("app.kubernetes.io/resource-sequence",
                NamingConvention.GetResourceSequence(ctx.ResourceSequence));

        return toreturn;
    }

    /// <summary>
    /// See example: https://github.com/pulumi/pulumi/issues/3956
    /// The output of this function is the base64 encoded dockerCfg
    /// </summary>
    /// <param name="loginServer"></param>
    /// <param name="adminUserName"></param>
    /// <param name="adminPassword"></param>
    /// <returns></returns>
    protected Output<string> GetPrivateRegistryConfig(Input<string> loginServer, Input<string> adminUserName,
        Input<string> adminPassword)
    {
        var dockerCfg = Output.All(loginServer, adminUserName, adminPassword).Apply(values =>
        {
            var r = new Dictionary<string, object>();
            var server = values[0];
            var username = values[1];
            var password = values[2];
            r[server] = new
            {
                email = "notneeded@acme.com",
                username,
                password
            };

            return r;
        });

        // Serialize & base64 encode the secret data. 
        return dockerCfg.Apply(x =>
            Convert.ToBase64String(Encoding.UTF8.GetBytes(System.Text.Json.JsonSerializer.Serialize(x))));
    }

    protected Secret CreateRegistrySecret(KubernetesEnvironmentContext ctx, CustomResourceOptions options,
        Input<string> @namespace,
        string name = "regcred") => new("regcred", new SecretArgs
    {
        Metadata = new ObjectMetaArgs
        {
            Name = name,
            Namespace = @namespace
            //Labels = labels
        },
        Type = "kubernetes.io/dockercfg",
        Data =
        {
            {
                ".dockercfg", GetPrivateRegistryConfig(
                    ctx.Config.Require("registry_server"),
                    ctx.Config.Require("registry_username"),
                    ctx.Config.RequireSecret("registry_password"))
            }
        },
    }, options);

    protected Namespace SetNamespace(CustomResourceOptions resourceOptions, string name, Namespace? @namespace = null) 
        => @namespace ?? Namespace.Get($"{name}defaultns", "default", resourceOptions);
}