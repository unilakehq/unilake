using YamlDotNet.Serialization;

namespace Unilake.Cli.Config.Unilake;

public class ProxyStorage : IConfigNode
{
    public string Section { get; } = "proxy-storage";
    
    [YamlMember(Alias = "enabled")]
    public bool Enabled { get; set; }

    public IEnumerable<ValidateResult> Validate(EnvironmentConfig config, IConfigNode? parentNode,
        params string[] checkProps)
    {
        return Enumerable.Empty<ValidateResult>();
    }
}