
namespace Unilake.Cli.Config;

public class Webapi : IValidate
{
    public bool Enabled { get; set; }
    public Postgresql? Postgresql { get; set; }

    public IEnumerable<ValidateResult> Validate(EnvironmentConfig config)
    {
        if(!Enabled)
            yield break;

        if(Postgresql == null)
            yield return new ValidateResult("Components.Unilake.Postgresql", "Postgresql is undefined");
        else 
            foreach(var err in Postgresql.Validate(config))
                yield return new ValidateResult("Components.Unilake.Postgresql", err.Error);        
    }
}