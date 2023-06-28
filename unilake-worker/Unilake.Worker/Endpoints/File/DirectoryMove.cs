using Unilake.Worker.Contracts.Requests.File;
using Unilake.Worker.Contracts.Responses.File;
using Unilake.Worker.Events.File;
using Unilake.Worker.Events.File.Types;
using Unilake.Worker.Processors.PreProcessor;
using Unilake.Worker.Services;

namespace Unilake.Worker.Endpoints.File;

public class DirectoryMove : Endpoint<DirectoryMoveRequest, FileActionResultResponse>
{
    private readonly IProcessManager _manager;

    public DirectoryMove(IProcessManager manager)
    {
        _manager = manager;
    }

    public override void Configure()
    {
        Post("/file/directory/move");
        Summary(s =>
        {
            s.Summary = "Move a directory";
            s.Description = "Moves an existing directory to a new location. Can also be used to rename a directory.";
            s.Responses[200] =
                "Directory move queued/processed successfully.";
        });
        PreProcessors(new RequestActivityTracker<DirectoryMoveRequest>());
    }

    public override async Task HandleAsync(DirectoryMoveRequest request, CancellationToken cancellationToken)
    {
        string processId = _manager.GenerateProcessId(new FileActionResultResponse
        {
            Message = "Directory move action queued"
        });

        DirectoryMoveTaskEvent eventDetails = request;
        eventDetails.SetProcessReferenceId(processId)
            .SetRunAsync(request.AsyncRequest)
            .SetOnCancelledMessage("Directory move action cancelled")
            .SetOnInProgressMessage("Directory move action in progress");

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