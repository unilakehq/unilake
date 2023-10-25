
namespace Unilake.Cli.Config;

public class Opensearch : IValidate
{
    public bool Enabled { get; set; }
    public bool SingleNode { get; set; } = true;
    public string? Host { get; set; }
    public int Port { get; set; } = 9200;

    public IEnumerable<(string section, string error)> Validate(EnvironmentConfig config)
    {
        if(Enabled == false)
            yield break;

        if(string.IsNullOrWhiteSpace(Host))
            yield return ("Cloud.OpenSearch.Host", "Host is undefined");
    }
}