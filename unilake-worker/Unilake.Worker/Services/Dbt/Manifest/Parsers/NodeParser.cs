using Unilake.Worker.Models.Dbt;

namespace Unilake.Worker.Services.Dbt.Manifest.Parsers;

public class NodeParser
{
    public Dictionary<string, NodeMetaData> CreateNodeMetaMap(
        string projectName,
        object[] nodesMap,
        string rootPath)
    {
        var modelMetaMap = new Dictionary<string, NodeMetaData>();

        if (nodesMap == null)
            return modelMetaMap;

        foreach (dynamic node in nodesMap)
        {
            string resourceType = node.resource_type;
            if (resourceType == DbtProject.ResourceTypeModel ||
                resourceType == DbtProject.ResourceTypeSeed ||
                resourceType == DbtProject.ResourceTypeSnapshot)
            {
                string name = node.name;
                string originalFilePath = node.original_file_path;
                string database = node.database;
                string schema = node.schema;
                string alias = node.alias;
                string packageName = node.package_name;

                string fullPath = ManifestParser.CreateFullPathForNode(projectName, rootPath, packageName, originalFilePath);

                if (string.IsNullOrEmpty(fullPath))
                {
                    continue;
                }

                modelMetaMap[name] = new NodeMetaData
                {
                    Path = fullPath,
                    Database = database,
                    Schema = schema,
                    Alias = alias,
                    PackageName = packageName
                };
            }
        }

        return modelMetaMap;
    }
}
