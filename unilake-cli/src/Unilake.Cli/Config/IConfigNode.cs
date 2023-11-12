namespace Unilake.Cli.Config;

public interface IConfigNode
{
    string Section { get; }
    IEnumerable<ValidateResult> Validate(EnvironmentConfig config, IConfigNode? parentNode, params string[] checkProps);
    public static bool CheckProp(string propName, string[] checkProps) =>
        checkProps.Length == 0 || (!checkProps.Contains(string.Empty) && checkProps.Contains(propName));
}
