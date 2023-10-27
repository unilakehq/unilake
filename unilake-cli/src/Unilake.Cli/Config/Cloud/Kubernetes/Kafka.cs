
namespace Unilake.Cli.Config;

public class Kafka : IValidate
{
    public bool Enabled { get; set; }
    public string? Server { get; set; }
    public string? SchemaRegistry { get; set; }

    public IEnumerable<ValidateResult> Validate(EnvironmentConfig config)
    {
        if(!Enabled)
            yield break;

        if(string.IsNullOrWhiteSpace(Server))
            yield return new ValidateResult("Cloud.Kafka.Server", "Server is undefined");
           
        if(string.IsNullOrWhiteSpace(SchemaRegistry))
            yield return new ValidateResult("Cloud.Kafka.SchemaRegistry", "SchemaRegistry is undefined");
    }
}