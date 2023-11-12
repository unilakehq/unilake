using System.ComponentModel.DataAnnotations;
using CommandLine;
using Spectre.Console;
using Unilake.Cli.Args;
using Parser = Unilake.Cli.Config.Parser;

namespace Unilake.Cli;

[Verb("validate", HelpText = "Validate a unilake config file")]
public class ValidateOptions : Options
{

    [Option('f', "file", Required = true, HelpText = "File to validate, defaults to unilake.default.yaml if not provided")]
    public required string FilePath { get; set; }

    public override Task<int> ExecuteAsync(CancellationToken cancellationToken)
    {
        if(!File.Exists(FilePath))
        {
            PrintErrorFoundHeader();
            AnsiConsole.MarkupLine(Message.ValidationConfigFileNotFound, FilePath);
            Console.WriteLine();
            return Task.FromResult(1);
        }

        var result = Parser.ParseFromPath(FilePath);
        if (!result.IsValid())
            result.PrettyPrintErrors();
        else
            PrintNoErrorsFoundHeader();
        
        return Task.FromResult(0);
    }

    private void PrintErrorFoundHeader() => AnsiConsole.MarkupLine(Message.ValiditionErrorsHeader);
    private void PrintNoErrorsFoundHeader() => AnsiConsole.MarkupLine(Message.ValidationNoErrorsHeader);
}
