
using Unilake.Cli.Config.Cloud;
using YamlDotNet.Serialization;

namespace Unilake.Cli.Config.Storage;

public sealed class Postgresql : IConfigNode
{
    public string Section { get; } = "postgresql";

    [YamlMember(Alias = "enabled")]
    public bool Enabled { get; set; }
    [YamlMember(Alias = "username")]
    public string? Username { get; set; }
    [YamlMember(Alias = "password")]
    public string? Password { get; set; }
    [YamlMember(Alias = "host")]
    public string? Host { get; set; }
    [YamlMember(Alias = "port")]
    public int Port { get; set; } = 5432;
    [YamlMember(Alias = "name")]
    public string? Name { get; set; }
    [YamlMember(Alias = "schema")]
    public string? Schema { get; set; }

    public IEnumerable<ValidateResult> Validate(EnvironmentConfig config, IConfigNode? parentNode,
        params string[] checkProps)
    {
        if (!Enabled)
            yield break;

        if (IConfigNode.CheckProp(nameof(Username), checkProps) && string.IsNullOrWhiteSpace(Username))
            yield return new ValidateResult(this, "username", "username is undefined");

        if (IConfigNode.CheckProp(nameof(Password), checkProps) && string.IsNullOrWhiteSpace(Password))
            yield return new ValidateResult(this, "password", "password is undefined");

        if (IConfigNode.CheckProp(nameof(Host), checkProps) && string.IsNullOrWhiteSpace(Host))
            yield return new ValidateResult(this, "host", "host is undefined");

        if (IConfigNode.CheckProp(nameof(Schema), checkProps) && string.IsNullOrWhiteSpace(Schema))
            yield return new ValidateResult(this, "schema", "schema is undefined");

        if (IConfigNode.CheckProp(nameof(Name), checkProps) && string.IsNullOrWhiteSpace(Name))
            yield return new ValidateResult(this, "name", "name is undefined");
    }
}