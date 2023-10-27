
namespace Unilake.Cli.Config;

public class Opensearch : IValidate
{
    public bool Enabled { get; set; }
    public bool SingleNode { get; set; } = true;
    public string? Host { get; set; }
    public int Port { get; set; } = 9200;

    public IEnumerable<ValidateResult> Validate(EnvironmentConfig config)
    {
        if(!Enabled)
            yield break;

        if(string.IsNullOrWhiteSpace(Host))
            yield return new ValidateResult("Cloud.OpenSearch.Host", "Host is undefined");
    }
}