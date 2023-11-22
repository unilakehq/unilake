using Pulumi;

namespace Unilake.Iac.Kubernetes.Custom;

// For more information, see: https://docs.starrocks.io/en-us/latest/administration/sr_operator
public class StarRockCluster : KubernetesComponentResource
{
    public StarRockCluster(KubernetesEnvironmentContext ctx, string name, ComponentResourceOptions? options = null, bool checkNamingConvention = true) 
        : base("unilake:kubernetes:custom:starrock", name, options, checkNamingConvention)
    {
        throw new NotImplementedException();
    }
}