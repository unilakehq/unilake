
using YamlDotNet.Serialization;

namespace Unilake.Cli.Config.Unilake;

public class Webapp : IConfigNode
{
    public string Section { get; } = "webapp";
    
    [YamlMember(Alias = "enabled")]
    public bool Enabled { get; set; }

    public IEnumerable<ValidateResult> Validate(EnvironmentConfig config, IConfigNode? parentNode)
    {
        return Enumerable.Empty<ValidateResult>();
    }
}