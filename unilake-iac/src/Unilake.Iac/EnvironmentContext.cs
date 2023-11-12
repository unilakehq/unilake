using Pulumi;

namespace Unilake.Iac;

public class EnvironmentContext
{
    public Config Config { get; } = new();

    public EnvironmentContext()
    {
        
    }

    protected EnvironmentContext(EnvironmentContext ctx)
    {
        Tenant = ctx.Tenant;
        Environment = ctx.Environment;
        CloudProvider = ctx.CloudProvider;
        CustomTags = ctx.CustomTags;
        Domain = ctx.Domain;
        Region = ctx.Region;
        EnvironmentSequence = ctx.EnvironmentSequence;
        ResourceSequence = ctx.ResourceSequence;
    }
    
    /// <summary>
    ///     Derive the environment context from an existing name
    /// </summary>
    /// <param name="name"></param>
    public EnvironmentContext(string name)
    {
        const int tenantIndex = 0;
        const int domainIndex = 1;
        const int environmentSequenceIndex = 2;
        // Skipping index 3 as it is the resource name
        const int environmentIndex = 4;
        const int cloudProviderIndex = 5;
        const int regionIndex = 6;
        const int resourceSequenceIndex = 7;

        var items = name.Split('-');
        if (items.Length < 8) // Or use another appropriate check for the expected length
            throw new ArgumentException($"The provided name '{name}' is not in the expected format.");

        Tenant = items[tenantIndex];
        Domain = items[domainIndex];

        if (!int.TryParse(items[environmentSequenceIndex], out var environmentSequence))
            throw new FormatException($"Environment sequence part '{items[environmentSequenceIndex]}' is not a valid integer.");
        EnvironmentSequence = environmentSequence;

        Environment = items[environmentIndex];
        CloudProvider = items[cloudProviderIndex];
        Region = items[regionIndex];

        if (!int.TryParse(items[resourceSequenceIndex], out var resourceSequence))
            throw new FormatException($"Resource sequence part '{items[resourceSequenceIndex]}' is not a valid integer.");
        ResourceSequence = resourceSequence;
    }


    public EnvironmentContext WithTenantName(string name)
    {
        Tenant = name;
        return this;
    }

    public EnvironmentContext WithEnvironmentSequence(int sequence)
    {
        EnvironmentSequence = sequence;
        return this;
    }

    public EnvironmentContext WithDomainName(string name)
    {
        Domain = name;
        return this;
    }

    public EnvironmentContext WithEnvironment(string environment)
    {
        Environment = environment;
        return this;
    }

    public EnvironmentContext WithCloudProvider(string cloudProvider)
    {
        CloudProvider = cloudProvider;
        return this;
    }

    public EnvironmentContext WithRegion(string name)
    {
        Region = name;
        return this;
    }

    public EnvironmentContext WithResourceSequence(int sequence)
    {
        ResourceSequence = sequence;
        return this;
    }

    public EnvironmentContext Copy()
    {
        return new EnvironmentContext
        {
            Domain = Domain,
            Environment = Environment,
            Region = Region,
            CloudProvider = CloudProvider,
            Tenant = Tenant,
            EnvironmentSequence = EnvironmentSequence,
            ResourceSequence = ResourceSequence,
            CustomTags = CustomTags
        };
    }

    public string Tenant { get; private set; } = string.Empty;

    public int EnvironmentSequence { get; private set; } = 1;

    public string Domain { get; private set; } = String.Empty;

    public string Environment { get; private set; } = "D";

    public string CloudProvider { get; private set; } = "KUBERNETES";

    public string Region { get; private set; } = "WE";

    public int ResourceSequence { get; private set; }

    public Dictionary<string, string> CustomTags { get; private set; } = new();
}