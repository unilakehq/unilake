
using YamlDotNet.Serialization;

namespace Unilake.Cli.Config.Dependencies;

public sealed class Starrocks : IConfigNode
{
    public string Section { get; } = "starrocks";
    
    [YamlMember(Alias = "enabled")]
    public bool Enabled { get; set; }

    public IEnumerable<ValidateResult> Validate(EnvironmentConfig config, IConfigNode? parentNode,
        params string[] checkProps)
    {
        return Enumerable.Empty<ValidateResult>();
    }
}