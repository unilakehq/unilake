using Unilake.Worker.Contracts.Responses;

namespace Unilake.Worker.Contracts;

public interface IRequestResponse
{
    public string ProcessReferenceId { get; set; }
    public ResultStatus Status { get; set; }
    public string Message { get; set; }
}