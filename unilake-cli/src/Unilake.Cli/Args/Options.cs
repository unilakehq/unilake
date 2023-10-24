using CommandLine;

namespace Unilake.Cli.Args;

public abstract class Options
{
    [Option('v', "verbose", Required = false, HelpText = "Set output to verbose messages.")]
    public bool Verbose { get; set; }

    public abstract int Execute();
}