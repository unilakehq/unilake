using Unilake.Worker.Contracts;
using Unilake.Worker.Contracts.Responses;

namespace Unilake.Worker.Tests.Endpoints.Git;

public abstract class GitEndpointTest
{
    protected T CreateResponse<T>(string message, string processReferenceId, ResultStatus status = ResultStatus.Queued)
    where T : IRequestResponse, new()
        => new ()
        {
            Status = status,
            Message = message,
            ProcessReferenceId = processReferenceId
        };
}