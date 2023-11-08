using System.Reflection;
using CommandLine;
using Spectre.Console;

namespace Unilake.Cli.Args;

[Verb("init", HelpText = "Initialize a new unilake configuration file.")]
public class InitOptions : Options
{
    public override async Task<int> ExecuteAsync(CancellationToken cancellationToken)
    {
        var currentDirectory = Directory.GetCurrentDirectory();
        await ExtractEmbeddedResourceAsync(currentDirectory, "Unilake.Cli.unilake.default.yaml", "unilake.yaml");
        AnsiConsole.MarkupLine(Message.GreenDone);
        return 0;
    }

    private async Task ExtractEmbeddedResourceAsync(string outputDirectory, string resourceLocation, string fileName)
    {
        await using Stream? stream = Assembly.GetExecutingAssembly().GetManifestResourceStream(resourceLocation);
        if (stream == null)
            throw new ArgumentException($"No resource found at {resourceLocation}");

        await using FileStream fileStream = new FileStream(Path.Combine(outputDirectory, fileName), FileMode.Create);
        await stream.CopyToAsync(fileStream);
    }
}