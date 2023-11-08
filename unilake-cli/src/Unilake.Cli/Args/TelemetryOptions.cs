using CommandLine;
using Spectre.Console;

namespace Unilake.Cli.Args;

[Verb("telemetry", HelpText = "Disable or enable (privatized) telemetry data")]
public class TelemetryOptions : Options
{

    [Option('e', "enabled", Required = false, HelpText = "Enable telemetry (default)")]
    public bool Enable { get; set; }

    [Option('d', "disable", Required = false, HelpText = "Disable telemetry")]
    public bool Disable { get; set; }
    
    public override Task<int> ExecuteAsync(CancellationToken cancellationToken)
    {
        AnsiConsole.MarkupLine(Message.TelemetryUnsupported);
        AnsiConsole.MarkupLine(string.Format(Message.CurrentState, ":cross_mark: [bold][red]Disabled[/][/]"));
        return Task.FromResult(0);
    }
}
