
namespace Unilake.Cli.Config;

public class Components : IValidate
{
    public Unilake? Unilake { get; set; }
    public Datahub? Datahub { get; set; }
    public Starrocks? Starrocks { get; set; }
    public Boxyhq? Boxyhq { get; set; }
    public Development? Development { get; set; }

    public IEnumerable<(string section, string error)> Validate(EnvironmentConfig config)
    {
        throw new NotImplementedException();
    }
}