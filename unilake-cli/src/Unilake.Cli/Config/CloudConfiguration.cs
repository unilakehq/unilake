
namespace Unilake.Cli.Config;

public class CloudConfiguration : IValidate
{
    public Kubernetes? Kubernetes { get; set; }

    public IEnumerable<(string section, string error)> Validate(EnvironmentConfig config)
    {
        if(Kubernetes == null)
            yield return ("cloud", "Kubernetes is undefined");
        else
            foreach (var err in Kubernetes.Validate(config))
                yield return err;
    }
}