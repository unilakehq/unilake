using System.Reflection;
using System.Text;
using CommandLine;
using CommandLine.Text;
using Spectre.Console;
using Unilake.Cli.Args;

namespace Unilake.Cli;

public static class Program
{
    public static async Task<int> Main(params string[] args)
    {
        var parser = new Parser(config => config.HelpWriter = null);
        var parserResult = parser.ParseArguments<UpOptions, DestroyOptions, InitOptions, TelemetryOptions, ValidateOptions>(args);
        var result = await parserResult.MapResult(
            (UpOptions opts) => RunAsync(opts),
            (DestroyOptions opts) => RunAsync(opts),
            (InitOptions opts) => RunAsync(opts),
            (TelemetryOptions opts) => RunAsync(opts),
            (ValidateOptions opts) => RunAsync(opts),
            errs => Task.FromResult(DisplayHelp(parserResult, errs))
        );
        return result;
    }

    private static async Task<int> RunAsync(Options option) => await option.ExecuteAsync(CancellationToken.None);

    private static int DisplayHelp<T>(ParserResult<T> result, IEnumerable<Error> errs)
    {
        var helpText = HelpText.AutoBuild(result, h =>
        {
            StringBuilder sb = new StringBuilder();
            var version = Assembly.GetExecutingAssembly().GetCustomAttributes(typeof(AssemblyInformationalVersionAttribute), false)[0] as AssemblyInformationalVersionAttribute;
            sb.Append($"UniLake CLI :rocket: - {version?.InformationalVersion} \n" +
                      "[bold][red]>>> THIS CLI IS UNDER DEVELOPMENT <<<[/][/]");
            if (errs.Any(x => x.Tag == ErrorType.NoVerbSelectedError))
                sb.Append("\n\nTo begin using this CLI, you need to run the following command: \n\n" +
                          "       $ unilake init \n\n" +
                          "This will generate a new unilake.yaml file in the current directory.");

            h.AdditionalNewLineAfterOption = false;
            h.Heading = sb.ToString();
            return HelpText.DefaultParsingErrorsHandler(result, h);
        }, e => e);

        helpText.Copyright = String.Empty;
        helpText.AdditionalNewLineAfterOption = false;
        AnsiConsole.Markup(helpText);
        return 0;
    }
}