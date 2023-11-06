
using Unilake.Cli.Config.Unilake;
using YamlDotNet.Serialization;

namespace Unilake.Cli.Config;

public class UnilakeConf : IConfigNode
{
    public string Section { get; } = "unilake";
    
    [YamlMember(Alias = "webapp")]
    public Webapp? Webapp { get; set; }
    [YamlMember(Alias = "webapi")]
    public Webapi? Webapi { get; set; }
    [YamlMember(Alias = "proxy-query")]
    public ProxyQuery? ProxyQuery { get; set; }
    [YamlMember(Alias = "proxy-storage")]
    public ProxyStorage? ProxyStorage { get; set; }

    public IEnumerable<ValidateResult> Validate(EnvironmentConfig config, IConfigNode? parentNode,
        params string[] checkProps)
    {
        foreach(var err in (Webapp?.Validate(config, this) ?? Enumerable.Empty<ValidateResult>())
            .Concat(Webapi?.Validate(config, this) ?? Enumerable.Empty<ValidateResult>())
            .Concat(ProxyQuery?.Validate(config, this) ?? Enumerable.Empty<ValidateResult>())
            .Concat(ProxyStorage?.Validate(config, this) ?? Enumerable.Empty<ValidateResult>()))
            yield return err.AddSection(this);
    }
}