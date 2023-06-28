using Unilake.Worker.Contracts.Requests.File;
using Unilake.Worker.Contracts.Responses.File;
using Unilake.Worker.Events.File;
using Unilake.Worker.Events.File.Types;
using Unilake.Worker.Processors.PreProcessor;
using Unilake.Worker.Services;

namespace Unilake.Worker.Endpoints.File;

public class DirectoryCreate : Endpoint<DirectoryCreateRequest, FileActionResultResponse>
{
    private readonly IProcessManager _manager;

    public DirectoryCreate(IProcessManager manager)
    {
        _manager = manager;
    }

    public override void Configure()
    {
        Post("/file/directory/create");
        Summary(s =>
        {
            s.Summary = "Creates a new directory";
            s.Description = "Creates a new directory in the specified directory.";
            s.Responses[200] =
                "Directory create queued/processed successfully.";
        });
        PreProcessors(new RequestActivityTracker<DirectoryCreateRequest>());
    }

    public override async Task HandleAsync(DirectoryCreateRequest request, CancellationToken cancellationToken)
    {
        string processId = _manager.GenerateProcessId(new FileActionResultResponse
        {
            Message = "Directory create action queued"
        });

        DirectoryCreateTaskEvent eventDetails = request;
        eventDetails.SetProcessReferenceId(processId)
            .SetRunAsync(request.AsyncRequest)
            .SetOnCancelledMessage("Directory create action cancelled")
            .SetOnInProgressMessage("Directory create action in progress");

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