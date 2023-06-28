using OneOf;
using OneOf.Types;
using Unilake.Worker.Contracts;
using Unilake.Worker.Contracts.Responses;

namespace Unilake.Worker.Services;

public class ProcessManager : IProcessManager
{
    private readonly FixedSizeConcurrentDictionary<string, IRequestResponse> _requests = new(100);

    public OneOf<Success<T>, Error<Exception>> Status<T>(string requestProcessReferenceId) 
        where T : class, IRequestResponse
    {
        if (!_requests.TryGetValue(requestProcessReferenceId, out var request))
            return new Error<Exception>(new Exception("Request not found"));
        if (request is T obj)
            return new Success<T>(obj);
        else
            return new Error<Exception>(new Exception("Request is not of expected type"));
    }

    public void SetResultStatus(string eventModelProcessReferenceId, ResultStatus status, string message = null)
    {
        if (_requests.TryGetValue(eventModelProcessReferenceId, out var req))
        {
            req.Status = status;
            if (!string.IsNullOrEmpty(message))
                req.Message = message;
        }
    }

    public void SetSuccessResponse(string eventModelProcessReferenceId, Success<IRequestResponse> success)
    {
        success.Value.Status = ResultStatus.Success;
        _requests.SetValue(eventModelProcessReferenceId, success.Value);
    }

    public void SetErrorResponse(string eventModelProcessReferenceId, Error<string> error)
    {
        if (_requests.TryGetValue(eventModelProcessReferenceId, out var req))
        {
            req.Status = ResultStatus.Error;
            req.Message = error.Value;
        }
    }

    public string GenerateProcessId(IRequestResponse request)
    {
        var guid = Guid.NewGuid().ToString();
        _requests.Add(guid, request);
        request.ProcessReferenceId = guid;
        request.Status = ResultStatus.Queued;
        return guid;
    }

    public OneOf<Success<IRequestResponse>, Error<Exception>> Cancel(string requestProcessReferenceId)
    {
        if (!_requests.TryGetValue(requestProcessReferenceId, out var request))
            return new Error<Exception>(new Exception("Request not found"));
        if (request.Status == ResultStatus.Cancelled)
            return new Error<Exception>(new Exception("Request is already cancelled"));
        if (request.Status != ResultStatus.Queued)
            return new Error<Exception>(new Exception("Request is not queued"));

        request.Status = ResultStatus.Cancelled;
        request.Message = "Pending cancellation";
        return new Success<IRequestResponse>(request);
    }

    public async Task PublishEventAsync<T>(T eventModel, Mode waitMode, CancellationToken cancellationToken) where T : IEvent
        => await eventModel.PublishAsync(waitMode, cancellationToken).ConfigureAwait(false);
}