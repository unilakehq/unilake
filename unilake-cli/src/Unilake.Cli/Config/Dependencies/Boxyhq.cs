
using Unilake.Cli.Config.Storage;
using YamlDotNet.Serialization;

namespace Unilake.Cli.Config;

public class Boxyhq : IConfigNode
{
    public string Section { get; } = "boxyhq";
    
    [YamlMember(Alias = "enabled")]
    public bool Enabled { get; set; }
    [YamlMember(Alias = "postgresql")]
    public Postgresql? Postgresql { get; set; }

    public IEnumerable<ValidateResult> Validate(EnvironmentConfig config, IConfigNode? parentNode,
        params string[] checkProps)
    {
        if(!Enabled)
            yield break;

        if(Postgresql == null)
            yield return new ValidateResult(this, "postgresql", "Postgresql and database information are missing");
        else
            foreach(var err in Postgresql.Validate(config, this))
                yield return err.AddSection(this);
    }
}