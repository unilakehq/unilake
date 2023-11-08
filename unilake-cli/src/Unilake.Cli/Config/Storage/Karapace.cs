
using YamlDotNet.Serialization;

namespace Unilake.Cli.Config.Storage;

public class Karapace : IConfigNode
{
    public string Section { get; } = "karapace";
    
    [YamlMember(Alias = "enabled")]
    public bool Enabled { get; set; }

    public IEnumerable<ValidateResult> Validate(EnvironmentConfig config, IConfigNode? parentNode,
        params string[] checkProps)
    {
        yield break;
    }
}