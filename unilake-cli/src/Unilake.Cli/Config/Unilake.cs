
namespace Unilake.Cli.Config;

public class Unilake : IValidate
{
    public Webapp? Webapp { get; set; }
    public Webapi? Webapi { get; set; }
    public ProxyQuery? ProxyQuery { get; set; }
    public ProxyStorage? ProxyStorage { get; set; }

    public IEnumerable<ValidateResult> Validate(EnvironmentConfig config)
    {
        return (Webapp?.Validate(config) ?? Enumerable.Empty<ValidateResult>())
            .Concat(Webapi?.Validate(config) ?? Enumerable.Empty<ValidateResult>())
            .Concat(ProxyQuery?.Validate(config) ?? Enumerable.Empty<ValidateResult>())
            .Concat(ProxyStorage?.Validate(config) ?? Enumerable.Empty<ValidateResult>());
    }
}