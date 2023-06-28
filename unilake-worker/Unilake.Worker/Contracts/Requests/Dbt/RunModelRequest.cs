namespace Unilake.Worker.Contracts.Requests.Dbt;

public class RunModelRequest
{
    public string ModelPath { get; set; }

    public string ModelType { get; set; }
}