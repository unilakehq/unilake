namespace Unilake.Iac.Kubernetes;

/// <summary>
/// TODO: https://releasehub.com/blog/10-kubernetes-namespace-best-practices-to-start-following#:~:text=The%20naming%20convention%20for%20the,characters%20can%20only%20be%20lowercase.
/// </summary>
public class NamingConventionKubernetesResource : NamingConvention
{
    public override (bool isSuccess, string errorMessage) IsCompliant(string name, string type)
    {
        // for deployment, should have a name and if needed a sequence number that follows name-xx where xx can be 01, 02 etc...
        return (true, "");
    }
    
    public string GetName(string name, EnvironmentContext ctx)
    {
        // Check input (tenant, max 8 chars)
        var tenant = GetTenant(ctx.Tenant);

        // Check input (sequence, max 9999)
        var sequence = GetEnvironmentSequence(ctx.EnvironmentSequence);

        // Check domain (domain, max 4 characters)
        var domain = GetDomain(ctx.Domain);

        // Check environment (environment, must be either D, T, A or P)
        var environment = GetEnvironment(ctx.Environment);

        // Check cloud provider (CloudProvider)
        var cloudProvider = GetCloudProvider(ctx.CloudProvider);

        // Check region (region, we)
        var region = GetRegion(ctx.Region);

        // True name
        var names = new List<string>
        {
            tenant, domain, sequence, name, environment, cloudProvider, region
        };

        // Check if we need to add a resource sequence
        if (ctx.ResourceSequence > 0)
            names.Add(GetResourceSequence(ctx.ResourceSequence));
        
        return string.Join("-", names.ToArray()).ToLower();
    }

    public override string GetAbbreviation<T>() => throw new NotImplementedException("abbreviation not supported for kubernetes deployments");
}