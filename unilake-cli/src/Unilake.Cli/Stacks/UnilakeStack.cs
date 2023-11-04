using Pulumi;
using Unilake.Cli.Config;

namespace Unilake.Cli;

public abstract class UnilakeStack : Stack
{
    protected EnvironmentConfig Config {get; set;}

    public abstract (string name, string version)[] Packages { get; }

    public UnilakeStack(EnvironmentConfig config)
    {
        Config = config;
    }

    public abstract Task Create();
}
