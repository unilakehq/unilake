
namespace Unilake.Cli.Config;

public class CloudConfiguration : IValidate
{
    public Kubernetes? Kubernetes { get; set; }

    public IEnumerable<ValidateResult> Validate(EnvironmentConfig config)
    {
        if(Kubernetes == null)
            yield return new ValidateResult("cloud", "Kubernetes is undefined");
        else
            foreach (var err in Kubernetes.Validate(config))
                yield return err;
    }
}