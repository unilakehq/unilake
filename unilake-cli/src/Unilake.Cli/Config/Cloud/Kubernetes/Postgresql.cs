
namespace Unilake.Cli.Config;

public class Postgresql : IValidate
{
    public bool Enabled { get; set; }
    public string? Username { get; set; }
    public string? Password { get; set; }
    public string? Host { get; set; }
    public int Port { get; set; } = 5432;
    public string? Name { get; set; }
    public string? Schema { get; set; }

    public IEnumerable<(string section, string error)> Validate(EnvironmentConfig config)
    {
        if(Enabled == false)
            yield break;
        
        if(string.IsNullOrWhiteSpace(Username))
            yield return ("Cloud.PostgreSql.Username", "Username is undefined");

        if(string.IsNullOrWhiteSpace(Host))
            yield return ("Cloud.PostgreSql.Host", "Host is undefined");

        if(string.IsNullOrWhiteSpace(Name))
            yield return ("Cloud.PostgreSql.Name", "Name is undefined (which is used as the name of the database)");

        if(string.IsNullOrWhiteSpace(Schema))
            yield return ("Cloud.PostgreSql.Schema", "Schema is undefined");
    }
}