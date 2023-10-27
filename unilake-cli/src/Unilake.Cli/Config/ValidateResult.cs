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
}
