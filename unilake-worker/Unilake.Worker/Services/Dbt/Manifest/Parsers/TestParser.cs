using Unilake.Worker.Models.Dbt;

namespace Unilake.Worker.Services.Dbt.Manifest.Parsers;

public class TestParser
{
    public Dictionary<string, TestMetaData> CreateTestMetaMap(
        object[] testsMap,
        string rootPath)
    {
        var testMetaMap = new Dictionary<string, TestMetaData>();

        if (testsMap == null)
            return testMetaMap;

        foreach (dynamic test in testsMap)
        {
            string resourceType = test.resource_type;
            if (resourceType == DbtProject.ResourceTypeTest)
            {
                string name = test.name;
                string rawSql = test.raw_sql;
                string originalFilePath = test.original_file_path;
                string database = test.database;
                string schema = test.schema;
                string alias = test.alias;
                string columnName = test.column_name;

                string fullPath = Path.Combine(rootPath, originalFilePath);
                testMetaMap[name] = new TestMetaData
                {
                    Path = fullPath,
                    RawSql = rawSql,
                    Database = database,
                    Schema = schema,
                    Alias = alias,
                    ColumnName = columnName
                };
            }
        }

        return testMetaMap;
    }
}
