using Spectre.Console;
using CliWrap;
using CliWrap.Buffered;

namespace Unilake.Cli;

internal static class StackContext
{
    public static async Task<(bool isSuccess, string dependency, string errorMessage)> CheckEnvironmentDependenciesAsync(Dictionary<string, string[]> dependencies)
    {
        try
        {
            foreach (var dep in dependencies)
            {
                var result = await CliWrap.Cli.Wrap(dep.Key).WithStandardErrorPipe(PipeTarget.Null)
                    .WithArguments(dep.Value)
                    .WithValidation(CommandResultValidation.None)
                    .ExecuteBufferedAsync();
                if (result.ExitCode != 0)
                    return (false, dep.Key, "Process returned an error: " + result.StandardError);
            }
        }
        catch (Exception exc)
        {
            Console.WriteLine(exc.Message);
        }
        return (true, string.Empty, string.Empty);
    }
    public static void ReportMissingDependency(string name, string guidance) => AnsiConsole.MarkupLine($"The following dependency could not be met {name}. \n\t{guidance}");
    public static void ReportFaultyDependency(string name, string errorMessage) => AnsiConsole.MarkupLine($"The following dependency ({name}) is in an error state. {errorMessage}");
    public static void ProcessCheckEnvironmentDependenciesResult((bool isSuccess, string dependency, string errorMessage) dep_result)
    {
        if (!dep_result.isSuccess)
        {
            if (!string.IsNullOrWhiteSpace(dep_result.errorMessage))
            {
                ReportFaultyDependency(dep_result.dependency, dep_result.errorMessage);
                return;
            }
            switch (dep_result.dependency)
            {
                case "pulumi":
                    ReportMissingDependency(dep_result.dependency, "Please install pulumi, see: https://www.pulumi.com/docs/install/");
                    break;
                case "kubectl":
                    ReportMissingDependency(dep_result.dependency, "Please install kubectl, see: https://kubernetes.io/docs/tasks/tools/");
                    break;
                default:
                    throw new CliException($"Unknown dependency provided {dep_result.dependency}");
            }
        }
    }
}
