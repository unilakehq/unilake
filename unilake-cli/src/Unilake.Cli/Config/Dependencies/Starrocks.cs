
using YamlDotNet.Serialization;

namespace Unilake.Cli.Config;

public class Starrocks : IConfigNode
{
    public string Section { get; } = "starrocks";
    
    [YamlMember(Alias = "enabled")]
    public bool Enabled { get; set; }

    public IEnumerable<ValidateResult> Validate(EnvironmentConfig config, IConfigNode? parentNode)
    {
        return Enumerable.Empty<ValidateResult>();
    }
}