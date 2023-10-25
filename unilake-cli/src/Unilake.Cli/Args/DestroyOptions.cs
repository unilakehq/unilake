using CommandLine;

namespace Unilake.Cli.Args;

[Verb("destroy", HelpText = "Destroy and remove all resources of a UniLake deployment.")]
public class DestroyOptions : Options
{
    public override Task<int> ExecuteAsync(CancellationToken cancellationToken)
    {
        throw new NotImplementedException();
    }
}