using CommandLine;
using Unilake.Cli.Stacks;

namespace Unilake.Cli.Args;

[Verb("destroy", HelpText = "Destroy and remove all resources of a UniLake deployment.")]
public class DestroyOptions : StackOptions
{
    public override async Task<int> ExecuteAsync(CancellationToken cancellationToken)
    {
        Console.WriteLine("Running destroy command...");

        // Get environment
        var parsed = await EnvironmentChecks();
        if (parsed.IsT1)
            return parsed.AsT1;

        // execute stack command
        var workspace = await new StackHandler<Kubernetes>(new Kubernetes(parsed.AsT0)).InitWorkspace("someProject", "someStack", cancellationToken);
        await workspace.InstallPluginsAsync(cancellationToken);
        var result = await workspace.DestroyAsync(cancellationToken);
        if (result == null)
            throw new CliException("Result cannot be null from Pulumi Automation");
        workspace.ReportDestroySummary(result);
        return 1;
    }
}