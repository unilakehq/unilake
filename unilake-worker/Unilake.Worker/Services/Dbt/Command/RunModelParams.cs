namespace Unilake.Worker.Services.Dbt.Command;

public class RunModelParams {
    public string PlusOperatorLeft { get; set; }
    public string ModelName { get; set; }
    public string PlusOperatorRight { get; set; }
}