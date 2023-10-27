
namespace Unilake.Cli.Config;

public class Gitea : IValidate
{
    public bool Enabled { get; set; }
    public string? AdminUsername { get; set; }
    public string? AdminPassword { get; set; }
    public string? AdminEmail { get; set; }
    public Postgresql? Postgresql { get; set; }
    public Redis? Redis { get; set; }

    public IEnumerable<ValidateResult> Validate(EnvironmentConfig config)
    {
        if(!Enabled)
            yield break;
        
        if(string.IsNullOrWhiteSpace(AdminUsername))
            yield return new ValidateResult("Components.Development.Gitea.AdminUsername", "AdminUsername is undefined");

        if(string.IsNullOrWhiteSpace(AdminPassword))
            yield return new ValidateResult("Components.Development.Gitea.AdminPassword", "AdminPassword is undefined");

        if(string.IsNullOrWhiteSpace(AdminEmail))
            yield return new ValidateResult("Components.Development.Gitea.AdminEmail", "AdminEmail is undefined");     

        foreach (var err in (Postgresql?.Validate(config) ?? Enumerable.Empty<ValidateResult>())
                            .Concat(Redis?.Validate(config) ?? Enumerable.Empty<ValidateResult>()))
            yield return err;
    }
}