using Unilake.Cli.Config.Storage;
using YamlDotNet.Serialization;

namespace Unilake.Cli.Config.Unilake;

public class Webapi : IConfigNode
{
    public string Section { get; }  = "webapi";
    
    [YamlMember(Alias = "enabled")]
    public bool Enabled { get; set; }
    [YamlMember(Alias = "postgresql")]
    public Postgresql? Postgresql { get; set; }

    public IEnumerable<ValidateResult> Validate(EnvironmentConfig config, IConfigNode? parentNode)
    {
        if(!Enabled)
            yield break;

        if(Postgresql == null)
            yield return new ValidateResult(this, "postgresql", "postgresql is undefined");
        else 
            foreach(var err in Postgresql.Validate(config, this))
                yield return err.AddSection(this);
    }
}