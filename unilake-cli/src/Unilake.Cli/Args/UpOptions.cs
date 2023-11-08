using CommandLine;
using Parser = Unilake.Cli.Config.Parser;

namespace Unilake.Cli.Args;

[Verb("up", HelpText = "Create or update a UniLake deployment.")]
public class UpOptions : Options
{
    [Option('c', "config-file", Required = false, HelpText = "Config file location.")]
    public string? ConfigFile { get; set; }

    [Option('f', "skip-preview", Required = false, HelpText = "Skip previewing changes before applying.")]
    public bool Force { get; set; } = false;

    public override async Task<int> ExecuteAsync(CancellationToken cancellationToken)
    {
        Console.WriteLine("Running up command...");
        string configFile = ConfigFile?? Path.Combine(Directory.GetCurrentDirectory(), "unilake.default.yaml");
        if (!File.Exists(configFile))
        {
            Console.Error.WriteLine($"Config file not found: {configFile}");
            return 1;
        }

        var parsed = Parser.ParseFromPath(configFile);
        if (!parsed.IsValid())
        {
            parsed.PrettyPrintErrors();
            return 1;
        }

        // TODO: see, https://github.com/pulumi/automation-api-examples/blob/main/dotnet/LocalProgram/automation/Program.cs
        var workspace = await new StackHandler<Kubernetes>(new Kubernetes(parsed)).InitWorkspace("someProject", "someStack", cancellationToken);
        await workspace.InstallPluginsAsync(cancellationToken);
        await workspace.UpAsync(cancellationToken);
        return 1;
    }
}