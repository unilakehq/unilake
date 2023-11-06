
using Unilake.Cli.Config.Cloud;
using Unilake.Cli.Config.Cloud.Kubernetes;
using Unilake.Cli.Config.Storage;
using YamlDotNet.Serialization;

namespace Unilake.Cli.Config;

public class Gitea : IConfigNode
{
    public string Section { get; } = "gitea";
    
    [YamlMember(Alias = "enabled")]
    public bool Enabled { get; set; }
    [YamlMember(Alias = "admin-username")]
    public string? AdminUsername { get; set; }
    [YamlMember(Alias = "admin-password")]
    public string? AdminPassword { get; set; }
    [YamlMember(Alias = "admin-email")]
    public string? AdminEmail { get; set; }
    [YamlMember(Alias = "postgresql")]
    public Postgresql? Postgresql { get; set; }
    [YamlMember(Alias = "redis")]
    public Redis? Redis { get; set; }

    public IEnumerable<ValidateResult> Validate(EnvironmentConfig config, IConfigNode? parentNode,
        params string[] checkProps)
    {
        if(!Enabled)
            yield break;
        
        if(IConfigNode.CheckProp(nameof(AdminUsername), checkProps) && string.IsNullOrWhiteSpace(AdminUsername))
            yield return new ValidateResult(this, "admin-username", "admin-username is undefined");

        if(IConfigNode.CheckProp(nameof(AdminPassword), checkProps) && string.IsNullOrWhiteSpace(AdminPassword))
            yield return new ValidateResult(this, "admin-password", "admin-password is undefined");

        if(IConfigNode.CheckProp(nameof(AdminEmail), checkProps) && string.IsNullOrWhiteSpace(AdminEmail))
            yield return new ValidateResult(this, "admin-email", "admin-email is undefined");

        foreach (var err in (Postgresql?.Validate(config, this) ?? Enumerable.Empty<ValidateResult>())
                 .Concat(Redis?.Validate(config, this) ?? Enumerable.Empty<ValidateResult>()))
            yield return err.AddSection(this);
    }
}