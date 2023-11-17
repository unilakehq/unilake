using Pulumi;
using Pulumi.Kubernetes.Core.V1;
using Pulumi.Kubernetes.Helm.V3;
using Pulumi.Kubernetes.Types.Inputs.Core.V1;
using Pulumi.Kubernetes.Types.Inputs.Helm.V3;
using Pulumi.Kubernetes.Types.Inputs.Meta.V1;
using Unilake.Iac.Kubernetes.Helm.Input;

namespace Unilake.Iac.Kubernetes.Helm;

public class OpenSearch : KubernetesComponentResource
{
    /// <summary>
    /// Service associated to this OpenSearch instance
    /// </summary>
    public Service @Service { get; private set; }

    /// <summary>
    /// Secret associated to this OpenSearch instance
    /// </summary>
    public Secret @Secret { get; private set; }

    public OpenSearch(KubernetesEnvironmentContext ctx, OpenSearchArgs inputArgs, Namespace? @namespace = null, 
        string name = "opensearch", ComponentResourceOptions? options = null, bool checkNamingConvention = true) 
            : base("unilake:kubernetes:helm:opensearch", name, options, checkNamingConvention)
    {
        // check input
        if (inputArgs == null) throw new ArgumentNullException(nameof(inputArgs));

        // Set default options
        var resourceOptions = CreateOptions(options);
        resourceOptions.Parent = this;
        resourceOptions.Provider = ctx.Provider;
        
        // Set namespace
        @namespace = SetNamespace(resourceOptions, name, @namespace);
        
        // Create secret for authentication
        var secret = new Secret(name, new SecretArgs{
            Metadata = new ObjectMetaArgs{
                Name = name,
                Namespace = @namespace.Metadata.Apply(x => x.Name),
            },           
        }, resourceOptions); 
       
        //Get OpenSearch chart and add details
        var releaseArgs = new ReleaseArgs
        {
            Name = name,
            Chart = "opensearch",
            Version = inputArgs.Version,
            Namespace = @namespace.Metadata.Apply(x => x.Name),
            RepositoryOpts = new RepositoryOptsArgs
            {
                Repo = "https://opensearch-project.github.io/helm-charts/"
            },
            Values = new InputMap<object> // https://github.com/opensearch-project/helm-charts/blob/main/charts/opensearch/values.yaml
            {
                ["singleNode"] = inputArgs.SingleNode,
                //["labels"] = GetLabels(ctx, "opensearch", "opensearch", "opensearch", inputArgs.Version)
            },
            // By default Release resource will wait till all created resources
            // are available. Set this to true to skip waiting on resources being
            // available.
            SkipAwait = false
        };

        // Check if a private registry is used
        if(inputArgs.UsePrivateRegsitry)
            throw new NotImplementedException("Private registry is not implemented yet");

        // Create the minio instance
        var opensearchInstance = new Release(name, releaseArgs, resourceOptions);
        
        // Get output
        var status = opensearchInstance.Status;
        @Service = Service.Get(name, Output.All(status).Apply(s => $"{s[0].Namespace}/{s[0].Name}-cluster-master"), resourceOptions);
        @Secret = secret;
    }
}