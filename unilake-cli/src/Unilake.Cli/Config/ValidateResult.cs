namespace Unilake.Cli;

public class ValidateResult
{
    public string Section { get; set; }
    public string Error { get; set; }

    public ValidateResult(string section, string error)
    {
        Section = section;
        Error = error;
    }
    
    public ValidateResult(IConfigNode node, string section, string error)
    {
        Section = node.Section + "->" + section;
        Error = error;
    }

    public ValidateResult AddSection(IConfigNode node) => AddSection(node.Section);
    
    public ValidateResult AddSection(string section)
    {
        Section = section + "->" + Section;
        return this;
    }
}
