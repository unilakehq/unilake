using CommandLine;
using Pulumi.Automation;
using Unilake.Cli.Config;

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
        // TODO: see, https://github.com/pulumi/automation-api-examples/blob/main/dotnet/LocalProgram/automation/Program.cs
        var stackArgs = new LocalProgramArgs("", "");
        var stack = await LocalWorkspace.CreateOrSelectStackAsync(stackArgs);
        
        var registration = cancellationToken.Register(() => stack.CancelAsync(CancellationToken.None));

        try
        {
            // TODO: create environment as set in the config
            var config = new EnvironmentConfig();
            
        }
        finally
        {
            await registration.DisposeAsync();
        }

        return 1;
    }
}