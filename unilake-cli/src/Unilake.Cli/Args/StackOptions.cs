using CommandLine;
using OneOf;
using Unilake.Cli.Args;
using Unilake.Cli.Config;
using Parser = Unilake.Cli.Config.Parser;

namespace Unilake.Cli;

public abstract class StackOptions : Options
{
    [Option('c', "config-file", Required = false, HelpText = "Config file location.")]
    public string? ConfigFile { get; set; }
    protected bool FileBased => !string.IsNullOrWhiteSpace(ConfigFile);

    protected async Task<OneOf<EnvironmentConfig, int>> EnvironmentChecks()
    {
        // check config file (presence)
        if (FileBased && !File.Exists(ConfigFile))
        {
            Console.Error.WriteLine($"Config file not found: {ConfigFile}");
            return 1;
        }

        // check config file (correctness)
        var parsed = FileBased ? Parser.ParseFromPath(ConfigFile!) : Parser.ParseFromEmbeddedResource("Unilake.Cli.unilake.default.yaml");
        if (!parsed.IsValid())
        {
            parsed.PrettyPrintErrors();
            return 1;
        }

        // check dependencies (kubectl, pulumi)
        var deps = new Dictionary<string, string[]>{
            {"pulumi", new[] {"version"}},
            {"kubectl", new[] {"version"}}
        };
        var dep_result = await StackContext.CheckEnvironmentDependenciesAsync(deps);

        StackContext.ProcessCheckEnvironmentDependenciesResult(dep_result);

        if (!dep_result.isSuccess)
            return 1;

        return parsed;
    }
}
