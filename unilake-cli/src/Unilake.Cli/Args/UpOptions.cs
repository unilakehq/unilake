using CommandLine;
using Unilake.Cli.Stacks;

namespace Unilake.Cli.Args;

[Verb("up", HelpText = "Create or update a UniLake deployment.")]
public class UpOptions : StackOptions
{
    [Option('f', "skip-preview", Required = false, HelpText = "Skip previewing changes before applying.")]
    public bool Force { get; set; } = false;

    public override async Task<int> ExecuteAsync(CancellationToken cancellationToken)
    {
        Console.WriteLine("Running up command...");

        // run stack
        var parsed = await EnvironmentChecks();
        if (parsed.IsT1)
            return parsed.AsT1;
        var workspace = await new StackHandler<Kubernetes>(new Kubernetes(parsed.AsT0)).InitWorkspace("someProject", "someStack", cancellationToken);
        await workspace.InstallPluginsAsync(cancellationToken);
        var result = await workspace.UpAsync(cancellationToken);
        if (result == null)
            throw new CliException("Result cannot be null from Pulumi Automation");
        workspace.ReportUpSummary(result);
        return 1;
    }
}