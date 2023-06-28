using Unilake.Worker.Contracts.Requests.File;
using Unilake.Worker.Contracts.Responses.File;
using Unilake.Worker.Events.File;
using Unilake.Worker.Events.File.Types;
using Unilake.Worker.Processors.PreProcessor;
using Unilake.Worker.Services;

namespace Unilake.Worker.Endpoints.File;

public class FileDelete : Endpoint<FileDeleteRequest, FileActionResultResponse>
{
    private readonly IProcessManager _manager;

    public FileDelete(IProcessManager manager)
    {
        _manager = manager;
    }

    public override void Configure()
    {
        Post("/file/delete");
        Summary(s =>
        {
            s.Summary = "Deletes an existing file";
            s.Description = "Deletes the specified file.";
            s.Responses[200] =
                "File delete queued/processed successfully.";
        });
        PreProcessors(new RequestActivityTracker<FileDeleteRequest>());
    }

    public override async Task HandleAsync(FileDeleteRequest request, CancellationToken cancellationToken)
    {
        string processId = _manager.GenerateProcessId(new FileActionResultResponse
        {
            Message = "File delete action queued"
        });

        FileDeleteTaskEvent eventDetails = request;
        eventDetails.SetProcessReferenceId(processId)
            .SetRunAsync(request.AsyncRequest)
            .SetOnCancelledMessage("File delete action cancelled")
            .SetOnInProgressMessage("File delete action in progress");

        await _manager.PublishEventAsync<FileTaskEvent>(eventDetails, request.GetMode(), cancellationToken).ConfigureAwait(false);
        await _manager.Status<FileActionResultResponse>(processId).Match(
            o => SendAsync(o.Value, cancellation: cancellationToken).ConfigureAwait(false),
            e =>
            {
                Logger.LogError(e.Value, CommonMessages.AnErrorOccuredWhileRetrievingTheEvent);
                AddError(e.Value.Message);
                return SendErrorsAsync(cancellation: cancellationToken).ConfigureAwait(false);
            }
        );
    }
}