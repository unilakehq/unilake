using Unilake.Worker.Contracts.Requests.Git;
using Unilake.Worker.Contracts.Responses.Git;
using Unilake.Worker.Events.Git;
using Unilake.Worker.Events.Git.Types;
using Unilake.Worker.Processors.PreProcessor;
using Unilake.Worker.Services;

namespace Unilake.Worker.Endpoints.Git;

public class Merge : Endpoint<GitBranchMergeRequest, GitActionResultResponse>
{
    private readonly IProcessManager _manager;

    public Merge(IProcessManager manager)
    {
        _manager = manager;
    }

    public override void Configure()
    {
        Post("/git/branch/merge");
        Summary(s =>
        {
            s.Summary = "Initiates a merge";
            s.Description = "Will initiate a merge to the given branch, from the current branch as set in the repo.";
            s.Responses[200] = "Git merge queued/processed successfully.";
        });
        PreProcessors(new RequestActivityTracker<GitBranchMergeRequest>());
    }

    public override async Task HandleAsync(GitBranchMergeRequest request, CancellationToken cancellationToken)
    {
        string processId = _manager.GenerateProcessId(new GitActionResultResponse
        {
            Message = "Git branch merge action queued"
        });

        GitBranchMergeTaskEvent eventDetails = new GitBranchMergeTaskEvent();
        eventDetails.SetProcessReferenceId(processId)
            .SetRunAsync(request.AsyncRequest)
            .SetOnCancelledMessage("Git merge action cancelled")
            .SetOnInProgressMessage("Git merge action in progress");

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