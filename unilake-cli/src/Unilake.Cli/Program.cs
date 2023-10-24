using CommandLine;
using CommandLine.Text;
using Unilake.Cli.Args;

var parserResult = Parser.Default.ParseArguments<UpOptions, DestroyOptions>(args);
int result = parserResult.MapResult(
    (UpOptions opts) => Run(opts),
    (DestroyOptions opts) => Run(opts),
    errs => DisplayHelp(parserResult, errs)
);
return result;

int Run(Options option) => option.Execute();

int DisplayHelp<T>(ParserResult<T> result, IEnumerable<Error> errs)
{
    var helpText = HelpText.AutoBuild(result, h =>
    {
        h.AdditionalNewLineAfterOption = true;
        h.Heading = "Your Custom Heading";
        return HelpText.DefaultParsingErrorsHandler(result, h);
    }, e => e);
    
    helpText.Copyright = "Your Custom Copyright Information";
    helpText.AdditionalNewLineAfterOption = false;

    Console.WriteLine(helpText);

    return 0;
}
