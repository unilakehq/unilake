
namespace Unilake.Cli.Config;

public class EnvironmentConfig
{
    public string? Version { get; set; }
    public CloudConfiguration? Cloud { get; set; }
    public Components? Components { get; set; }

    private static readonly string[] AllowedVersions = { "unilake.com/v1alpha1" };

    public IEnumerable<(string section, string error)> Validate()
    {
        if (string.IsNullOrWhiteSpace(Version) || !AllowedVersions.Contains(Version.ToLower()))
            yield return ("config", $"current version is {Version}, versions allowed are: {string.Join(", ", AllowedVersions)}");

        if (Components == null)
            yield return ("config", "Components must be configured");
        else
            foreach (var item in Components.Validate(this))
                yield return item;

        if(Cloud != null)
            foreach(var item in Cloud.Validate(this))
                yield return item;
    }
}