using Unilake.Worker.Contracts.Requests.File;
using Unilake.Worker.Contracts.Responses.File;
using Unilake.Worker.Events.File;
using Unilake.Worker.Events.File.Types;
using Unilake.Worker.Processors.PreProcessor;
using Unilake.Worker.Services;

namespace Unilake.Worker.Endpoints.File;

public class DirectoryDelete : Endpoint<DirectoryDeleteRequest, FileActionResultResponse>
{
    private readonly IProcessManager _manager;

    public DirectoryDelete(IProcessManager manager)
    {
        _manager = manager;
    }

    public override void Configure()
    {
        Post("/file/directory/delete");
        Summary(s =>
        {
            s.Summary = "Deletes an existing directory";
            s.Description = "Deletes the specified directory.";
            s.Responses[200] =
                "Directory delete queued/processed successfully.";
        });
        PreProcessors(new RequestActivityTracker<DirectoryDeleteRequest>());
    }

    public override async Task HandleAsync(DirectoryDeleteRequest request, CancellationToken cancellationToken)
    {
        string processId = _manager.GenerateProcessId(new FileActionResultResponse
        {
            Message = "Directory delete action queued"
        });

        DirectoryDeleteTaskEvent eventDetails = request;
        eventDetails.SetProcessReferenceId(processId)
            .SetRunAsync(request.AsyncRequest)
            .SetOnCancelledMessage("Directory delete action cancelled")
            .SetOnInProgressMessage("Directory delete action in progress");

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