using Pulumi;
using Pulumi.Kubernetes.Core.V1;
using Pulumi.Kubernetes.Helm.V3;
using Pulumi.Kubernetes.Types.Inputs.Core.V1;
using Pulumi.Kubernetes.Types.Inputs.Helm.V3;
using Pulumi.Kubernetes.Types.Inputs.Meta.V1;
using Unilake.Iac.Kubernetes.Helm.Input;

namespace Unilake.Iac.Kubernetes.Helm;

public class Minio : KubernetesComponentResource
{
    /// <summary>
    /// Service associated to this Minio instance
    /// </summary>
    public Service @Service { get; private set; }

    /// <summary>
    /// Secret associated to this Minio instance
    /// </summary>
    public Secret @Secret { get; private set; }

    public Output<string> RootUser { get; private set; }
    
    public Output<string> RootPassword { get; private set; }

    public Minio(KubernetesEnvironmentContext ctx, Namespace? @namespace = null, MinioArgs? inputArgs = null, 
        string name = "minio", ComponentResourceOptions? options = null, bool checkNamingConvention = true) 
            : base("pkg:kubernetes:helm:minio", name, options, checkNamingConvention)
    {
        // Check input
        inputArgs ??= new MinioArgs();

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
            Data = new InputMap<string>
            {
                {"rootUser", inputArgs.RootUser.Apply(x => x.EncodeBase64())},
                {"rootPassword", inputArgs.RootPassword.Apply(x => x.EncodeBase64())},
            }
        }, resourceOptions); 
       
        //Get Minio chart and add details
        var releaseArgs = new ReleaseArgs
        {
            Name = name,
            Chart = "minio",
            Version = inputArgs.Version,
            Namespace = @namespace.Metadata.Apply(x => x.Name),
            RepositoryOpts = new RepositoryOptsArgs
            {
                Repo = "https://charts.min.io/"
            },
            Values = new InputMap<object> // https://github.com/minio/minio/blob/master/helm/minio/values.yaml
            {
                ["mode"] = inputArgs.Replicas == 1 ? "standalone" : "distributed",
                ["existingSecret"] = secret.Metadata.Apply(x=> x.Name),
                ["replicas"] = inputArgs.Replicas,
                ["persistence"] = new Dictionary<string, object>
                {
                    ["enabled"] = true,
                    ["size"] = "25Gi"
                },
                ["resources"] = new Dictionary<string, object>
                {
                    ["requests"] = new Dictionary<string, object>
                    {
                        ["cpu"] = "1250m",
                        ["memory"] = "2Gi"
                    }
                },
                ["additionalLabels"] = GetLabels(ctx, "minio", "minio", "minio", inputArgs.Version)
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
            releaseArgs.Values.Add("image.repository", releaseArgs.Values.Apply(x => PrivateRegistryBase + x["image.repository"] ));
            releaseArgs.Values.Add("mcImage.repository", releaseArgs.Values.Apply(x => PrivateRegistryBase + x["mcImage.repository"] ));
        }

        // Check if initial buckets need to be created
        if(inputArgs.Buckets.Length > 0)
            releaseArgs.Values.Add("buckets", inputArgs.Buckets.Select(x => new InputMap<object>
            {
                ["name"] = x.Name,
                ["policy"] = x.Policy,
                ["purge"] = x.Purge,
                ["versioning"] = x.Versioning,
                ["objectlocking"] = x.ObjectLocking
            }).ToArray());

        // Create the minio instance
        var minioInstance = new Release(name, releaseArgs, resourceOptions);
        
        // Get output
        var status = minioInstance.Status;
        @Service = Service.Get(name, Output.All(status).Apply(s => $"{s[0].Namespace}/{s[0].Name}"), resourceOptions);
        @Secret = secret;
        RootUser = inputArgs.RootUser;
        RootPassword = inputArgs.RootPassword;
    }
}