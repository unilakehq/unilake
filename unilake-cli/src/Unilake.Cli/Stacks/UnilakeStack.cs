using OneOf;
using OneOf.Types;
using Unilake.Cli.Config;

namespace Unilake.Cli;

public abstract class UnilakeStack
{
    protected EnvironmentConfig Config {get; set;}

    public abstract (string name, string version)[] Packages { get; }

    public UnilakeStack(EnvironmentConfig config)
    {
        Config = config;
    }

    public abstract Task<OneOf<Success, Error<Exception>>> Create();
}
