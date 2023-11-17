
using YamlDotNet.Serialization;

namespace Unilake.Cli.Config;

public sealed class Target : IConfigNode
{
    public string Section { get; } = "target";
    
    [YamlMember(Alias = "cloud")]
    public string? Cloud { get; set; }

    public IEnumerable<ValidateResult> Validate(EnvironmentConfig config, IConfigNode? parentNode,
        params string[] checkProps)
    {
        if(string.IsNullOrWhiteSpace(Cloud))
            yield return new ValidateResult(this, "cloud", "cloud is undefined");
    }
}