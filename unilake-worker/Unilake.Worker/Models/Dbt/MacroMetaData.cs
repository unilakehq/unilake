namespace Unilake.Worker.Models.Dbt;

public class MacroMetaData
{
    public string Path { get; set; }
    public int Line { get; set; }
    public int Character { get; set; }
}