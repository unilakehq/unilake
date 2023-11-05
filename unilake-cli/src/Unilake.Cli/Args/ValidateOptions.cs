using CommandLine;
using Spectre.Console;
using Unilake.Cli.Args;

namespace Unilake.Cli;

[Verb("validate", HelpText = "Validate a unilake config file")]
public class ValidateOptions : Options
{

    [Option('f', "file", Required = true, HelpText = "File to validate, defaults to unilake.default.yaml if not provided")]
    public required string FilePath { get; set; }

    public override Task<int> ExecuteAsync(CancellationToken cancellationToken)
    {
        AnsiConsole.MarkupLine(Message.ValiditionErrorsHeader);
        if(!File.Exists(FilePath))
        {
            AnsiConsole.MarkupLineInterpolated(Message.ValidationConfigFileNotFound, FilePath);
        }
        return Task.FromResult(0);
    }
}
