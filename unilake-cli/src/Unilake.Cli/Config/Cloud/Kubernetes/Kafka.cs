
namespace Unilake.Cli.Config;

public class Kafka : IValidate
{
    public bool Enabled { get; set; }
    public string? Server { get; set; }
    public string? SchemaRegistry { get; set; }

    public IEnumerable<(string section, string error)> Validate(EnvironmentConfig config)
    {
        if(Enabled == false)
            yield break;

        if(string.IsNullOrWhiteSpace(Server))
            yield return ("Cloud.Kafka.Server", "Server is undefined");
           
        if(string.IsNullOrWhiteSpace(SchemaRegistry))
            yield return ("Cloud.Kafka.SchemaRegistry", "SchemaRegistry is undefined");
    }
}