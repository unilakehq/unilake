using Unilake.Cli.Config;

namespace Unilake.Cli;

public interface IConfigNode
{
    string Section { get; }
    IEnumerable<ValidateResult> Validate(EnvironmentConfig config, IConfigNode? parentNode);
}
