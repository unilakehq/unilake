namespace Unilake.Iac;

public abstract class NamingConvention
{
    public abstract (bool isSuccess, string errorMessage) IsCompliant(string name, string type);
    
    public abstract string GetAbbreviation<T>();
    
    protected string GetTenant(string tenant)
        => tenant.Length switch
        {
            > 8 => throw new FormatException("Tenant lenght must be less than 8 characters, but got " + tenant.Length),
            <= 8 => tenant + string.Join("", Enumerable.Range(0, 8 - tenant.Length).Select(x => "x"))
        };

    public string GetEnvironmentSequence(int sequence)
        => sequence switch
        {
            <= 0 => throw new FormatException("Sequence must be greater than 0, but current value is not"),
            > 9999 => throw new FormatException("Sequence must be less than 10000, but current value is " +
                                                sequence),
            <= 9999 => string.Join("", Enumerable.Range(0, 4 - sequence.ToString().Length).Select(x => "0")) + sequence
        };

    public string GetResourceSequence(int sequence)
        => sequence switch
        {
            <= 0 => throw new FormatException("Sequence must be greater than 0, but current value is not"),
            > 9999 => throw new FormatException(
                "Resource sequence must be less than 10000, but current value is " +
                sequence),
            <= 9999 => string.Join("", Enumerable.Range(0, 4 - sequence.ToString().Length).Select(x => "0")) + sequence
        };

    protected string GetDomain(string domain)
        => domain.Length switch
        {
            > 4 => throw new FormatException("Domain length must be less than 4 characters, but current value is " +
                                             domain),
            <= 4 => domain + string.Join("", Enumerable.Range(0, 4 - domain.Length).Select(x => "x"))
        };

    protected string GetEnvironment(string environment)
        => new[] { "D", "T", "A", "P" }.Contains(environment.ToUpper())
            ? environment.ToUpper()
            : throw new FormatException("Environment must be either D, T, A or P, but found " + environment);

    protected string GetCloudProvider(string cloudProvider)
        => new[] { "K8S" }.Contains(cloudProvider.ToUpper())
            ? cloudProvider.ToUpper()
            : throw new FormatException(
                "CloudProvider must be K8S (for Kubernetes), but current value is " +
                cloudProvider);

    protected string GetRegion(string region)
        => new[] { "WE" }.Contains(region.ToUpper())
            ? region.ToUpper()
            : throw new FormatException("Region must be WE, but current value is " + region);
}