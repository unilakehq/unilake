using Unilake.Cli.Config;

namespace Unilake.Cli;

public interface IConfigNode
{
    string Section { get; }
    // TODO: 1 add params so you can specify which items are required
    IEnumerable<ValidateResult> Validate(EnvironmentConfig config, IConfigNode? parentNode);
}
