using Unilake.Worker.Contracts.Requests.File;
using Unilake.Worker.Contracts.Responses.File;
using Unilake.Worker.Events.File;
using Unilake.Worker.Events.File.Types;
using Unilake.Worker.Processors.PreProcessor;
using Unilake.Worker.Services;

namespace Unilake.Worker.Endpoints.File;

public class FileMove : Endpoint<FileMoveRequest, FileActionResultResponse>
{
    private readonly IProcessManager _manager;

    public FileMove(IProcessManager manager)
    {
        _manager = manager;
    }

    public override void Configure()
    {
        Post("/file/move");
        Summary(s =>
        {
            s.Summary = "Move a file";
            s.Description = "Moves an existing file to a new location. Can also be used to rename a file.";
            s.Responses[200] =
                "File move queued/processed successfully.";
        });
        PreProcessors(new RequestActivityTracker<FileMoveRequest>());
    }

    public override async Task HandleAsync(FileMoveRequest request, CancellationToken cancellationToken)
    {
        string processId = _manager.GenerateProcessId(new FileActionResultResponse
        {
            Message = "File move action queued"
        });

        FileMoveTaskEvent eventDetails = request;
        eventDetails.SetProcessReferenceId(processId)
            .SetRunAsync(request.AsyncRequest)
            .SetOnCancelledMessage("File move action cancelled")
            .SetOnInProgressMessage("File move action in progress");

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