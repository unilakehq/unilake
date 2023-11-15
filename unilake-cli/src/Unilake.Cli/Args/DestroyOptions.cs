using CommandLine;
using Unilake.Cli.Stacks;
using Parser = Unilake.Cli.Config.Parser;

namespace Unilake.Cli.Args;

[Verb("destroy", HelpText = "Destroy and remove all resources of a UniLake deployment.")]
public class DestroyOptions : Options
{
    [Option('c', "config-file", Required = false, HelpText = "Config file location.")]
    public string? ConfigFile { get; set; }
    
    private bool FileBased => !string.IsNullOrWhiteSpace(ConfigFile);

    public override async Task<int> ExecuteAsync(CancellationToken cancellationToken)
    {
        Console.WriteLine("Running destroy command...");
        if (FileBased && !File.Exists(ConfigFile))
        {
            Console.Error.WriteLine($"Config file not found: {ConfigFile}");
            return 1;
        }

        var parsed = FileBased
            ? Parser.ParseFromPath(ConfigFile!)
            : Parser.ParseFromEmbeddedResource("Unilake.Cli.unilake.default.yaml");
        if (!parsed.IsValid())
        {
            parsed.PrettyPrintErrors();
            return 1;
        }

        var workspace =
            await new StackHandler<Kubernetes>(new Kubernetes(parsed)).InitWorkspace("someProject", "someStack",
                cancellationToken);
        await workspace.InstallPluginsAsync(cancellationToken);
        await workspace.DestroyAsync(cancellationToken);
        return 1;
    }
}