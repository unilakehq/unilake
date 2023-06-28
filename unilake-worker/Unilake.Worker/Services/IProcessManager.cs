using OneOf;
using OneOf.Types;
using Unilake.Worker.Contracts;
using Unilake.Worker.Contracts.Responses;

namespace Unilake.Worker.Services;

public interface IProcessManager
{
    OneOf<Success<T>, Error<Exception>> Status<T>(string requestProcessReferenceId) where T : class, IRequestResponse;
    void SetResultStatus(string eventModelProcessReferenceId, ResultStatus status, string message = null);
    void SetSuccessResponse(string eventModelProcessReferenceId, Success<IRequestResponse> success);
    void SetErrorResponse(string eventModelProcessReferenceId, Error<string> error);
    string GenerateProcessId(IRequestResponse request);
    OneOf<Success<IRequestResponse>, Error<Exception>> Cancel(string processReferenceId);
    Task PublishEventAsync<T>(T eventModel, Mode waitMode, CancellationToken cancellationToken) where T : IEvent;
}