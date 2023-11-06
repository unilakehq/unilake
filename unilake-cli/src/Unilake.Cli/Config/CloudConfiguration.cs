
using Unilake.Cli.Config.Cloud;
using YamlDotNet.Serialization;

namespace Unilake.Cli.Config;

public class CloudConfiguration : IConfigNode
{
    public string Section { get; } = "cloud";
    
    [YamlMember(Alias = "kubernetes")]
    public KubernetesConf? Kubernetes { get; set; }

    public IEnumerable<ValidateResult> Validate(EnvironmentConfig config, IConfigNode? parentNode,
        params string[] checkProps)
    {
        if(Kubernetes == null)
            yield return new ValidateResult(this, "kubernetes", "kubernetes is undefined");
        else
            foreach (var err in Kubernetes.Validate(config, this))
                yield return err.AddSection(this);
    }
}