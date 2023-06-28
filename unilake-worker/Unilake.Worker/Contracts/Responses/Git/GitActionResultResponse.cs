namespace Unilake.Worker.Contracts.Responses.Git;

public class GitActionResultResponse : IRequestResponse
{
    public ResultStatus Status { get; set; }
    public string Message { get; set; }
    public string ProcessReferenceId { get; set; }
}