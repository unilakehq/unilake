using Pulumi;
using Unilake.Iac.Kubernetes.Custom.Crds.StarRock.V1Alpha1;
using Unilake.Iac.Kubernetes.Custom.Crds.StarRock.V1Alpha1.Inputs;

namespace Unilake.Iac.Kubernetes.Custom;

// For more information, see: https://docs.starrocks.io/en-us/latest/administration/sr_operator
public class StarRockCluster : KubernetesComponentResource
{
    private readonly StarRockCluster _starRockCluster;
    
    public StarRockCluster(KubernetesEnvironmentContext ctx, string name, ComponentResourceOptions? options = null, bool checkNamingConvention = true) 
        : base("pkg:kubernetes:custom:starrock", name, options, checkNamingConvention)
    {
        // Set default options
        var resourceOptions = CreateOptions(options);
        resourceOptions.Parent = this;
        resourceOptions.Provider = ctx.Provider;
        
        // create args
        var ags = new StarRocksClusterArgs
        {
            Spec = new StarRocksClusterSpecArgs
            {
                StarRocksBeSpec = new StarRocksClusterSpecStarrocksbespecArgs
                {
                    
                }
            }
        };
        
        // set resource
        //_starRockCluster = new Crds.StarRock.V1Alpha1.StarRocksCluster(name, ags, resourceOptions);
    }
}