namespace Unilake.Worker.Models.Dbt;

public class DocMetaData
{
    public string Path { get; set; }
    public int Line { get; set; }
    public int Character { get; set; }
}