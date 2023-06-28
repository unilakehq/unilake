using Unilake.Worker.Models.Dbt;

namespace Unilake.Worker.Services.Dbt.Manifest.Parsers;

public class SourceParser
{ 
    public Dictionary<string, SourceMetaData> CreateSourceMetaMap(
        object[] sourcesMap,
        string rootPath)
    {
        var sourceMetaMap = new Dictionary<string, SourceMetaData>();

        if (sourcesMap == null)
            return sourceMetaMap;

        foreach (dynamic source in sourcesMap)
        {
            string resourceType = source.resource_type;
            if (resourceType == DbtProject.ResourceTypeSource)
            {
                string sourceName = source.source_name;
                string name = source.name;
                string originalFilePath = source.original_file_path;

                if (!sourceMetaMap.TryGetValue(sourceName, out SourceMetaData sourceMetaData))
                {
                    sourceMetaData = new SourceMetaData { Tables = new List<SourceTable>() };
                    sourceMetaMap[sourceName] = sourceMetaData;
                }

                string fullPath = Path.Combine(rootPath, originalFilePath);
                sourceMetaData.Tables.Add(new SourceTable { Name = name, Path = fullPath });
            }
        }

        return sourceMetaMap;
    }
}
