namespace Unilake.Worker.Models.Dbt;

public class TestMetaData
{
    public string Path { get; set; }
    public string Database { get; set; }
    public string Schema { get; set; }
    public string Alias { get; set; }
    public string RawSql { get; set; }
    public string ColumnName { get; set; }
}