namespace Unilake.Worker.Models.Dbt;

public class NodeMetaData
{
    public string Path { get; set; }
    public string Database { get; set; }
    public string Schema { get; set; }
    public string Alias { get; set; }
    public string PackageName { get; set; }
}