
using Unilake.Cli.Config.Cloud;
using YamlDotNet.Serialization;

namespace Unilake.Cli.Config.Storage;

public class Postgresql : IConfigNode
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

    public IEnumerable<ValidateResult> Validate(EnvironmentConfig config, IConfigNode? parentNode)
    {
        if(!Enabled)
            yield break;
        
        if(string.IsNullOrWhiteSpace(Username))
            yield return new ValidateResult(this, "username", "username is undefined");
        
        if(string.IsNullOrWhiteSpace(Password))
            yield return new ValidateResult(this, "password", "password is undefined");
        
        if(parentNode is not KubernetesConf && string.IsNullOrWhiteSpace(Host))
            yield return new ValidateResult(this, "host", "host is undefined");
        
        if(parentNode is Gitea && string.IsNullOrWhiteSpace(Schema))
            yield return new ValidateResult(this, "schema", "schema is undefined");
    }
}