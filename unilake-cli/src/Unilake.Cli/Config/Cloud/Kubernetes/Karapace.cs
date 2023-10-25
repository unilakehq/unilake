
namespace Unilake.Cli.Config;

public class Karapace : IValidate
{
    public bool Enabled { get; set; }

    public IEnumerable<(string section, string error)> Validate(EnvironmentConfig config)
    {
        throw new NotImplementedException();
    }
}