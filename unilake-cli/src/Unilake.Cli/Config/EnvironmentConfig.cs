
namespace Unilake.Cli.Config;

public class EnvironmentConfig
{
    public string? Version { get; set; }
    public CloudConfiguration? Cloud { get; set; }
    public Components? Components { get; set; }

    private static readonly string[] AllowedVersions = { "unilake.com/v1alpha1" };

    private ValidateResult[]? Errors;

    private IEnumerable<ValidateResult> Check()
    {
        if (string.IsNullOrWhiteSpace(Version) || !AllowedVersions.Contains(Version.ToLower()))
            yield return new ValidateResult("config", $"current version is {Version ?? "undefined"}, versions allowed are: {string.Join(", ", AllowedVersions)}");

        if (Components == null)
            yield return new ValidateResult("config", "Components must be configured");
        else
            foreach (var item in Components.Validate(this))
                yield return item;

        if(Cloud != null)
            foreach(var item in Cloud.Validate(this))
                yield return item;
    }

    public EnvironmentConfig Validate()
    {
        Errors ??= Check().ToArray();
        return this;
    }

    public ValidateResult[] GetErrors()
    {
        Errors ??= Check().ToArray();
        return Errors;
    }

    public bool IsValid()
    {
        if(Errors == null)
            this.Validate();

        return Errors?.Length == 0;
    }
}