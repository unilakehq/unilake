
using YamlDotNet.Serialization;

namespace Unilake.Cli.Config.Development;

public sealed class Pgweb : IConfigNode
{
    public string Section { get; } = "pgweb";
    
    [YamlMember(Alias = "enabled")]
    public bool Enabled { get; set; }
    [YamlMember(Alias = "target")]
    public Target? Target { get; set; }
    [YamlMember(Alias = "database")]
    public string? Database { get; set; }

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
        
        if(string.IsNullOrWhiteSpace(Database))
            yield return new ValidateResult(this, Section, "database is undefined");
    }
}