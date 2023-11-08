
using Spectre.Console;
using YamlDotNet.Serialization;

namespace Unilake.Cli.Config;

public class EnvironmentConfig
{
    [YamlMember(Alias = "version")]
    public string? Version { get; set; }
    
    [YamlMember(Alias = "cloud")]
    public CloudConfiguration? Cloud { get; set; }
    
    [YamlMember(Alias = "components")]
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
            foreach (var item in Components.Validate(this, null!))
                yield return item;

        if (Cloud == null) yield break;
        {
            foreach(var item in Cloud.Validate(this, null!))
                yield return item;
        }
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
            Validate();

        return Errors?.Length == 0;
    }

    public void PrettyPrintErrors()
    {
        Errors ??= Check().ToArray();
        if (Errors.Length == 0)
            return;
        
        AnsiConsole.MarkupLine(Message.ValiditionErrorsHeader);

        foreach (var error in Errors)
            AnsiConsole.MarkupLine(Message.ValidtionErrorMessage , error.Section, error.Error);
        
        Console.WriteLine();
        Console.WriteLine();
    }
}