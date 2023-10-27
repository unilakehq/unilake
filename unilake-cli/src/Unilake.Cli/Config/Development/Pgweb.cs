
namespace Unilake.Cli.Config;

public class Pgweb : IValidate
{
    public bool Enabled { get; set; }
    public Target? Target { get; set; }

    public IEnumerable<ValidateResult> Validate(EnvironmentConfig config)
    {
        if(!Enabled)
            yield break;

        if(Target == null)
            yield return new ValidateResult("Components.Development.Pgweb.Target", "Target is undefined");
        else 
            foreach(var err in Target.Validate(config))
                yield return new ValidateResult("Components.Development.Pgweb.Target", err.Error);
    }
}