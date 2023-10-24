using CommandLine;

namespace Unilake.Cli.Args;

[Verb("destroy", HelpText = "Destroy and remove all resources of a UniLake deployment.")]
public class DestroyOptions : Options
{
    public override int Execute()
    {
        Console.WriteLine("Running destroy command...");
        return 1;
    }
}