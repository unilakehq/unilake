using CommandLine;
using Unilake.Cli.Stacks;
using Parser = Unilake.Cli.Config.Parser;

namespace Unilake.Cli.Args;

[Verb("up", HelpText = "Create or update a UniLake deployment.")]
public class UpOptions : Options
{
    [Option('c', "config-file", Required = false, HelpText = "Config file location.")]
    public string? ConfigFile { get; set; }

    [Option('f', "skip-preview", Required = false, HelpText = "Skip previewing changes before applying.")]
    public bool Force { get; set; } = false;

    private bool FileBased => !string.IsNullOrWhiteSpace(ConfigFile);

    public override async Task<int> ExecuteAsync(CancellationToken cancellationToken)
    {
        Console.WriteLine("Running up command...");
        if (FileBased && !File.Exists(ConfigFile))
        {
            Console.Error.WriteLine($"Config file not found: {ConfigFile}");
            return 1;
        }

        var parsed = FileBased ? Parser.ParseFromPath(ConfigFile!) : Parser.ParseFromEmbeddedResource("Unilake.Cli.unilake.default.yaml");
        if (!parsed.IsValid())
        {
            parsed.PrettyPrintErrors();
            return 1;
        }
        
        var workspace = await new StackHandler<Kubernetes>(new Kubernetes(parsed)).InitWorkspace("someProject", "someStack", cancellationToken);
        await workspace.InstallPluginsAsync(cancellationToken);
        var result = await workspace.UpAsync(cancellationToken);
        if (result == null)
            throw new CliException("Result cannot be null from Pulumi Automation");
        workspace.ReportUpSummary(result);
        return 1;
    }
}