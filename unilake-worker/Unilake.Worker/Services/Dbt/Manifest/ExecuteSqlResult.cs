namespace Unilake.Worker.Services.Dbt.Manifest;

public class ExecuteSqlResult
{
    public Table Table { get; set; }
    public string RawSql { get; set; }
    public string CompiledSql { get; set; }
}