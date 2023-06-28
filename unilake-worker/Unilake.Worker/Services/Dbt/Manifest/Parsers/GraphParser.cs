using Unilake.Worker.Models.Dbt;

namespace Unilake.Worker.Services.Dbt.Manifest.Parsers;

public class GraphParser
{
    public GraphMetaMap CreateGraphMetaMap(
        Dictionary<string, List<string>> parentMap,
        Dictionary<string, List<string>> childrenMap,
        Dictionary<string, NodeMetaData> nodeMetaMap,
        Dictionary<string, SourceMetaData> sourceMetaMap,
        Dictionary<string, TestMetaData> testMetaMap)
    {
        List<string> Unique(List<string> nodes) => nodes.Distinct().ToList();

        var parents = parentMap.ToDictionary(
            entry => entry.Key,
            entry => new NodeGraphMetaData
            {
                Nodes = Unique(entry.Value)
                    .Select(MapToNode(sourceMetaMap, nodeMetaMap, testMetaMap))
                    .Where(node => node != null)
                    .ToList()
            });

        var children = childrenMap.ToDictionary(
            entry => entry.Key,
            entry => new NodeGraphMetaData
            {
                Nodes = Unique(entry.Value)
                    .Select(MapToNode(sourceMetaMap, nodeMetaMap, testMetaMap))
                    .Where(node => node != null && !(node is Test))
                    .ToList()
            });

        var tests = childrenMap.ToDictionary(
            entry => entry.Key,
            entry => new NodeGraphMetaData
            {
                Nodes = Unique(entry.Value)
                    .Select(MapToNode(sourceMetaMap, nodeMetaMap, testMetaMap))
                    .Where(node => node != null && node is Test)
                    .ToList()
            });

        return new GraphMetaMap
        {
            Parents = parents,
            Children = children,
            Tests = tests
        };
    }

    private Func<string, Node> MapToNode(
        Dictionary<string, SourceMetaData> sourceMetaMap,
        Dictionary<string, NodeMetaData> nodeMetaMap,
        Dictionary<string, TestMetaData> testMetaMap)
    {
        return parentNodeName =>
        {
            var nodeTypeAndName = parentNodeName.Split('.').ToList();
            string nodeType = nodeTypeAndName[0];
            string nodePackage = nodeTypeAndName[1];
            string nodeName = string.Join(".", nodeTypeAndName.Skip(2));

            switch (nodeType)
            {
                case "source":
                    string[] sourceNameAndTableName = nodeName.Split('.');
                    string sourceName = sourceNameAndTableName[0];
                    string tableName = sourceNameAndTableName[1];
                    if (sourceMetaMap.TryGetValue(sourceName, out var sourceMetaData))
                    {
                        string url = sourceMetaData.Tables.FirstOrDefault(table => table.Name == tableName)?.Path;
                        return new Source($"{tableName} ({sourceName})", parentNodeName, url);
                    }
                    break;
                case "model":
                    if (nodeMetaMap.TryGetValue(nodeName, out var nodeMetaData))
                        return new Model(nodeName, parentNodeName, nodeMetaData.Path);
                    break;
                case "seed":
                    if (nodeMetaMap.TryGetValue(nodeName, out nodeMetaData))
                        return new Seed(nodeName, parentNodeName, nodeMetaData.Path);
                    break;
                case "test":
                    if (testMetaMap.TryGetValue(nodeName.Split('.')[0], out var testMetaData))
                        return new Test(nodeName, parentNodeName, testMetaData.Path ?? string.Empty);
                    break;
                case "analysis":
                    if (nodeMetaMap.TryGetValue(nodeName, out nodeMetaData))
                        return new Analysis(nodeName, parentNodeName, nodeMetaData.Path);
                    break;
                case "snapshot":
                    if (nodeMetaMap.TryGetValue(nodeName, out nodeMetaData))
                        return new Snapshot(nodeName, parentNodeName, nodeMetaData.Path);
                    break;
                case "exposure":
                    if (nodeMetaMap.TryGetValue(nodeName, out nodeMetaData))
                        return new Exposure(nodeName, parentNodeName, nodeMetaData.Path);
                    break;
                default:
                    // TODO: some improved form of error handling!
                    Console.WriteLine($"Node Type '{nodeType}' not implemented!");
                    return null;
            }

            return null;
        };
    }
}
