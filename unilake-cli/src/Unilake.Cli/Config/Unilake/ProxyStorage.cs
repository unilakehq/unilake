namespace Unilake.Cli.Config;

public class ProxyStorage : IValidate
{
    public bool Enabled { get; set; }

    public IEnumerable<ValidateResult> Validate(EnvironmentConfig config)
    {
        return Enumerable.Empty<ValidateResult>();
    }
}