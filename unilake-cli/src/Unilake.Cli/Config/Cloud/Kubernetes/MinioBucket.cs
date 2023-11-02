
using YamlDotNet.Serialization;

namespace Unilake.Cli.Config.Cloud.Kubernetes;

public class MinioBucket : IConfigNode
{
    public string Section { get; } = "buckets";
    
    [YamlMember(Alias = "name")]
    public string? Name { get; set; }
    [YamlMember(Alias = "policy")]
    public string? Policy { get; set; }
    [YamlMember(Alias = "purge")]
    public bool Purge { get; set; } = false;
    [YamlMember(Alias = "versioning")]
    public bool Versioning { get; set; } = false;
    [YamlMember(Alias = "object-locking")]
    public bool ObjectLocking { get; set; } = false;

    

    public IEnumerable<ValidateResult> Validate(EnvironmentConfig config, IConfigNode? parentNode)
    {
        if(string.IsNullOrWhiteSpace(Name))
            yield return new ValidateResult(this, "name", "name is undefined");
        
        if(string.IsNullOrWhiteSpace(Policy))
            yield return new ValidateResult(this, "policy", "policy is undefined");
    }
}