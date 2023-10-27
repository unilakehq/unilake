
namespace Unilake.Cli.Config;

public class Redis : IValidate
{
    public bool Enabled { get; set; }
    public string? Host { get; set; }
    public int Database { get; set; } = 0;

    public IEnumerable<ValidateResult> Validate(EnvironmentConfig config)
    {
        if(!Enabled)
            yield break;
        
        if(string.IsNullOrWhiteSpace(Host))
            yield return new ValidateResult("Cloud.PostgreSql.Host", "Host is undefined");

        if(Database < 0)
            yield return new ValidateResult("Cloud.Redis.Database", "Database cannot be lower than 0");        
    }
}