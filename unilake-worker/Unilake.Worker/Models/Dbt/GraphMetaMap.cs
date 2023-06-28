namespace Unilake.Worker.Models.Dbt;

public class GraphMetaMap
{
    public Dictionary<string, NodeGraphMetaData> Parents { get; set; }
    public Dictionary<string, NodeGraphMetaData> Children { get; set; }
    public Dictionary<string, NodeGraphMetaData> Tests { get; set; }
}