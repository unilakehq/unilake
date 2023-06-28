using Unilake.Worker.Contracts.Requests.Git;
using Unilake.Worker.Contracts.Responses.Git;
using Unilake.Worker.Events.Git;
using Unilake.Worker.Events.Git.Types;
using Unilake.Worker.Processors.PreProcessor;
using Unilake.Worker.Services;

namespace Unilake.Worker.Endpoints.Git;

public class AbortMerge : Endpoint<GitAbortMergeRequest, GitActionResultResponse>
{
    private readonly IProcessManager _manager;

    public AbortMerge(IProcessManager manager)
    {
        _manager = manager;
    }

    public override void Configure()
    {
        Post("/git/branch/merge/abort");
        Summary(s =>
        {
            s.Summary = "Aborts a merge";
            s.Description = "In case a merge is in progress, this endpoint will abort the current merge.";
            s.Responses[200] = "Git abort merge queued/processed successfully.";
        });
        PreProcessors(new RequestActivityTracker<GitAbortMergeRequest>());
    }

    public override async Task HandleAsync(GitAbortMergeRequest request, CancellationToken cancellationToken)
    {
        string processId = _manager.GenerateProcessId(new GitActionResultResponse
        {
            Message = "Git abort branch merge action queued"
        });

        GitAbortMergeTaskEvent eventDetails = new GitAbortMergeTaskEvent();
        eventDetails.SetProcessReferenceId(processId)
            .SetRunAsync(request.AsyncRequest)
            .SetOnCancelledMessage("Git abort merge action cancelled")
            .SetOnInProgressMessage("Git abort merge action in progress");

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
