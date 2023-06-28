namespace Unilake.Worker.Contracts.Responses.File;

public class FileActionResultResponse : IRequestResponse
{
    public ResultStatus Status { get; set; }
    public string Message { get; set; }
    public string ProcessReferenceId { get; set; }
}