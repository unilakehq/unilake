namespace Unilake.Worker.Contracts.Responses.Dbt;

public class DbtActionResultResponse : IRequestResponse
{
    public ResultStatus Status { get; set; }
    public string Message { get; set; }
    public string ProcessReferenceId { get; set; }
}