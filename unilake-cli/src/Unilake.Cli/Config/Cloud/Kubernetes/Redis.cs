
namespace Unilake.Cli.Config;

public class Redis : IValidate
{
    public bool Enabled { get; set; }
    public string? Host { get; set; }
    public int? Database { get; set; }

    public IEnumerable<(string section, string error)> Validate(EnvironmentConfig config)
    {
        throw new NotImplementedException();
    }
}