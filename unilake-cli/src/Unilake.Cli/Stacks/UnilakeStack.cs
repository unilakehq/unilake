using OneOf;
using OneOf.Types;
using Unilake.Cli.Config;

namespace Unilake.Cli.Stacks;

public abstract class UnilakeStack
{
    protected EnvironmentConfig Config {get; set;}

    public abstract (string name, string version)[] Packages { get; }

    protected UnilakeStack(EnvironmentConfig config)
    {
        Config = config;
    }

    public abstract Task<OneOf<Success, Error<Exception>>> Create();
}
