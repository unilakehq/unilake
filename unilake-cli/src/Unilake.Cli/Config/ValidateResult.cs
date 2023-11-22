using Unilake.Cli.Config;

namespace Unilake.Cli;

public sealed class ValidateResult
{
    public string Section { get; set; }
    public string Error { get; set; }
    private const string Seperator = "->";

    public ValidateResult(string section, string error)
    {
        Section = section;
        Error = error;
    }
    
    public ValidateResult(IConfigNode node, string section, string error)
    {
        Section = node.Section + Seperator + section;
        Error = error;
    }

    public ValidateResult AddSection(IConfigNode node) => AddSection(node.Section);
    
    public ValidateResult AddSection(string section)
    {
        Section = section + Seperator + Section;
        return this;
    }
}
