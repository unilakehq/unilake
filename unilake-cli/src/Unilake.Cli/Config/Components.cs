
using Unilake.Cli.Config.Dependencies;
using YamlDotNet.Serialization;

namespace Unilake.Cli.Config;

public class Components : IConfigNode
{
    public string Section { get; } = "components";
    
    [YamlMember(Alias = "unilake")]
    public UnilakeConf? Unilake { get; set; }
    [YamlMember(Alias = "datahub")]
    public Datahub? Datahub { get; set; }
    [YamlMember(Alias = "starrocks")]    
    public Starrocks? Starrocks { get; set; }
   [YamlMember(Alias = "nessie")]    
    public Nessie? Nessie { get; set; }
    [YamlMember(Alias = "boxyhq")]
    public Boxyhq? Boxyhq { get; set; }
    [YamlMember(Alias = "development")]
    public Development? Development { get; set; }

    public IEnumerable<ValidateResult> Validate(EnvironmentConfig config, IConfigNode? parentNode,
        params string[] checkProps)
    {
        foreach (var err in (Unilake?.Validate(config, this) ?? Enumerable.Empty<ValidateResult>())
                 .Concat(Datahub?.Validate(config, this) ?? Enumerable.Empty<ValidateResult>())
                 .Concat(Starrocks?.Validate(config, this) ?? Enumerable.Empty<ValidateResult>())
                 .Concat(Boxyhq?.Validate(config, this) ?? Enumerable.Empty<ValidateResult>())
                 .Concat(Nessie?.Validate(config, this) ?? Enumerable.Empty<ValidateResult>())
                 .Concat(Development?.Validate(config, this) ?? Enumerable.Empty<ValidateResult>()))
            yield return err.AddSection(this);

    }
}