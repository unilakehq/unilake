
namespace Unilake.Cli.Config;

public class Target : IValidate
{
    public string? Cloud { get; set; }

    public IEnumerable<ValidateResult> Validate(EnvironmentConfig config)
    {
        if(string.IsNullOrWhiteSpace(Cloud))
            yield return new ValidateResult("Target", "Target is undefined");
    }
}