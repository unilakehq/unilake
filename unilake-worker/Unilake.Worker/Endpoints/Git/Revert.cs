using Unilake.Worker.Contracts.Requests.Git;
using Unilake.Worker.Contracts.Responses.Git;
using Unilake.Worker.Events.Git;
using Unilake.Worker.Events.Git.Types;
using Unilake.Worker.Processors.PreProcessor;
using Unilake.Worker.Services;

namespace Unilake.Worker.Endpoints.Git;

public class Revert : Endpoint<GitRevertRequest, GitActionResultResponse>
{
    private readonly IProcessManager _manager;

    public Revert(IProcessManager manager)
    {
        _manager = manager;
    }

    public override void Configure()
    {
        Post("/git/revert");
        Summary(s =>
        {
            s.Summary = "Revert changes locally";
            s.Description = "Revert changes done locally and no longer required.";
            s.Responses[200] = "Git revert action queued/processed successfully.";
        });
        PreProcessors(new RequestActivityTracker<GitRevertRequest>());
    }

    public override async Task HandleAsync(GitRevertRequest request, CancellationToken cancellationToken)
    {
        string processId = _manager.GenerateProcessId(new GitActionResultResponse
        {
            Message = "Git revert action queued"
        });

        GitRevertTaskEvent eventDetails = request;
        eventDetails.SetProcessReferenceId(processId)
            .SetRunAsync(request.AsyncRequest)
            .SetOnCancelledMessage("Git revert action cancelled")
            .SetOnInProgressMessage("Git revert action in progress");

        await _manager.PublishEventAsync<GitTaskEvent>(eventDetails, request.GetMode(), cancellationToken).ConfigureAwait(false);
        await _manager.Status<GitActionResultResponse>(processId).Match(
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