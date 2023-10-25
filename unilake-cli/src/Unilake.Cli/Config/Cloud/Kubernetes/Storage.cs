
namespace Unilake.Cli.Config;

public class Storage : IValidate
{
    public Minio? Minio { get; set; }

    public IEnumerable<(string section, string error)> Validate(EnvironmentConfig config)
    {
        throw new NotImplementedException();
    }
}