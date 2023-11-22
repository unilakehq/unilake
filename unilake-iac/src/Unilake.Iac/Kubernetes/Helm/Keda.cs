using Pulumi;
using Pulumi.Kubernetes.Core.V1;
using Pulumi.Kubernetes.Helm.V3;
using Pulumi.Kubernetes.Types.Inputs.Helm.V3;
using Unilake.Iac.Kubernetes.Helm.Input;

namespace Unilake.Iac.Kubernetes.Helm;

/// <summary>
/// TODO: create CRDs for scaling objects
/// </summary>
public sealed class Keda : KubernetesComponentResource
{
    public Keda(KubernetesEnvironmentContext ctx, KedaArgs inputArgs, Namespace? @namespace = null, 
        string name = "keda", ComponentResourceOptions? options = null, bool checkNamingConvention = true)
        : base("unilake:kubernetes:helm:keda", name, options, checkNamingConvention)
    {
        // Might be best to use the chart for this one
        // see: https://keda.sh/docs/2.8/deploy/
        // what will be needed, is a scaled object spec, which is a custom type to deploy : https://keda.sh/docs/2.8/concepts/scaling-deployments/#overview

        // check input
        if (inputArgs == null) throw new ArgumentNullException(nameof(inputArgs));
        
        // set default options
        var resourceOptions = CreateOptions(options);
        resourceOptions.Parent = this;
        resourceOptions.Provider = ctx.Provider;

        // Set namespace
        @namespace = SetNamespace(resourceOptions, name, @namespace);

        //Get Keda chart and add
        var releaseArgs = new ReleaseArgs
        {
            Name = name,
            Chart = "keda",
            Version = inputArgs.Version,
            Namespace = @namespace.Metadata.Apply(x => x.Name),
            RepositoryOpts = new RepositoryOptsArgs
            {
                Repo = "https://kedacore.github.io/charts"
            },
            Values = new InputMap<object> // https://github.com/kedacore/charts/blob/main/keda/values.yaml
            {
                ["podLabels"] = new Dictionary<string, object>
                {
                    ["keda"] = GetLabels(ctx, "keda", "keda", "keda", inputArgs.Version),
                    ["metricsAdapter"] = GetLabels(ctx, "keda", "keda", "metricsadapter", inputArgs.Version)
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
            releaseArgs.Values.Add("imagePullSecrets", new [] {registrySecret.Metadata.Apply(x => x.Name)});
            string PrivateRegistryBase = !string.IsNullOrWhiteSpace(inputArgs.PrivateRegistryBase) ? inputArgs.PrivateRegistryBase + "/" : "";
            releaseArgs.Values.Add("image.keda.repository", releaseArgs.Values.Apply(x => PrivateRegistryBase + x["image.keda.repository"] ));
            releaseArgs.Values.Add("image.metricsApiServer.repository", releaseArgs.Values.Apply(x => PrivateRegistryBase + x["image.metricsApiServer.repository"] ));
        }

        // Keda instance
        var kedaInstance = new Release(name, releaseArgs, resourceOptions);

        // Get output
        var _ = kedaInstance.Status;
    }
}