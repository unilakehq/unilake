using YamlDotNet.Serialization;

namespace Unilake.Cli.Config;

public class RedisUi : IConfigNode
{
    public string Section { get; } = "redis-ui";
    
    [YamlMember(Alias = "enabled")]
    public bool Enabled { get; set; }
    [YamlMember(Alias = "target")]
    public Target? Target { get; set; }

    public IEnumerable<ValidateResult> Validate(EnvironmentConfig config, IConfigNode? parentNode,
        params string[] checkProps)
    {
        if(!Enabled)
            yield break;

        if(Target == null)
            yield return new ValidateResult(this, "target", "target is undefined");
        else
            foreach (var err in Target.Validate(config, this))
                yield return err.AddSection(this);
    }
}