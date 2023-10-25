using Pulumi;

namespace Unilake.Iac;

public class EnvironmentContext
{
    public Config Config { get; } = new();
    
    public EnvironmentContext()
    {
        
    }

    protected EnvironmentContext(EnvironmentContext ctx) => new EnvironmentContext(ctx.Domain, ctx.Region,
        ctx.CloudProvider, ctx.Environment, ctx.EnvironmentSequence, ctx.Tenant, ctx.ResourceSequence, ctx.CustomTags);

    private EnvironmentContext(string domain = "", string region = "", string cloudProvider = "",
        string environment = "",
        int environmentSequence = 0, string tenant = "", int resourceSequence = 0,
        Dictionary<string, string>? customTags = null)
    {
        Domain = string.IsNullOrWhiteSpace(domain) ? Domain : domain;
        Environment = string.IsNullOrWhiteSpace(environment) ? Environment : environment;
        Region = string.IsNullOrWhiteSpace(region) ? Region : region;
        EnvironmentSequence = environmentSequence > 0 ? environmentSequence : EnvironmentSequence;
        Tenant = string.IsNullOrWhiteSpace(tenant) ? Tenant : tenant;
        CloudProvider = string.IsNullOrWhiteSpace(cloudProvider) ? CloudProvider : cloudProvider;
        ResourceSequence = resourceSequence > 0 ? resourceSequence : ResourceSequence;
        CustomTags = customTags ?? CustomTags;
    }

    /// <summary>
    ///     Derive the environment context from an existing name
    /// </summary>
    /// <param name="name"></param>
    public EnvironmentContext(string name)
    {
        var items = name.Split('-');
        for (var i = 0; i < items.Length; i++)
            switch (i)
            {
                case 0:
                    Tenant = items[i];
                    break;
                case 1:
                    Domain = items[i];
                    break;
                case 2:
                    EnvironmentSequence = int.Parse(items[i]);
                    break;
                case 3:
                    // Skip, is resource name
                    break;
                case 4:
                    Environment = items[i];
                    break;
                case 5:
                    CloudProvider = items[i];
                    break;
                case 6:
                    Region = items[i];
                    break;
                case 7:
                    ResourceSequence = int.Parse(items[i]);
                    break;
                default:
                    throw new Exception("Could not derive object based on name");
            }
    }

    public string Tenant { get; set; } = "INTERNAL";

    public int EnvironmentSequence { get; set; } = 1;

    public string Domain { get; set; } = "CORE";

    public string Environment { get; set; } = "D";

    public string CloudProvider { get; set; } = "AZURE";

    public string Region { get; set; } = "EUNL";

    public int ResourceSequence { get; set; } = 0;

    public Dictionary<string, string> CustomTags { get; set; } = new();

    /// <summary>
    /// Create a deep copy of this object, also allows you to make changes where needed
    /// </summary>
    /// <returns></returns>
    public EnvironmentContext Copy(string domain = "", string region = "", string cloudProvider = "",
        string environment = "",
        int environmentSequence = 0, string tenant = "", int resourceSequence = 0,
        Dictionary<string, string>? customTags = null) =>
        new(domain, region, cloudProvider, environment, environmentSequence, tenant, resourceSequence, customTags);
}