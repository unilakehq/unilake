
using YamlDotNet.Serialization;

namespace Unilake.Cli.Config.Unilake;

public sealed class Webapp : IConfigNode
{
    public string Section { get; } = "webapp";
    
    [YamlMember(Alias = "enabled")]
    public bool Enabled { get; set; }

    public IEnumerable<ValidateResult> Validate(EnvironmentConfig config, IConfigNode? parentNode,
        params string[] checkProps)
    {
        return Enumerable.Empty<ValidateResult>();
    }
}