namespace Unilake.Worker.Services.Dbt.Manifest;

public class Table
{
    public List<string> ColumnNames { get; set; }
    public List<List<object>> Rows { get; set; }
}