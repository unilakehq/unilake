
using YamlDotNet.Serialization;

namespace Unilake.Cli.Config.Cloud.Kubernetes;

public sealed class Minio : IConfigNode
{
    public string Section { get; } = "minio";
    
    [YamlMember(Alias = "enabled")]
    public bool Enabled { get; set; }
    [YamlMember(Alias = "root-user")]
    public string? RootUser { get; set; }
    [YamlMember(Alias = "root-password")]
    public string? RootPassword { get; set; }
    [YamlMember(Alias = "replicas")]
    public int Replicas { get; set; } = 1;
    [YamlMember(Alias = "buckets")]
    public List<MinioBucket>? Buckets { get; set; }

    public IEnumerable<ValidateResult> Validate(EnvironmentConfig config, IConfigNode? parentNode,
        params string[] checkProps)
    {
        if(!Enabled)
            yield break;

        if(IConfigNode.CheckProp(nameof(RootUser), checkProps) && string.IsNullOrWhiteSpace(RootUser))
            yield return new ValidateResult(this, "root-user", "root-user is undefined");
           
        if(IConfigNode.CheckProp(nameof(RootPassword), checkProps) && string.IsNullOrWhiteSpace(RootPassword))
            yield return new ValidateResult(this, "root-password", "root-password is undefined");

        if(IConfigNode.CheckProp(nameof(Replicas), checkProps) && Replicas < 1)
            yield return new ValidateResult(this, "replicas", "replicas cannot be below 1");

        if(Buckets == null || !Buckets.Any())
            yield return new ValidateResult(this, "buckets", "buckets cannot be below 1");
        else
            foreach(var err in Buckets.SelectMany(x => x.Validate(config, this)))
                yield return err;
        
    }
}