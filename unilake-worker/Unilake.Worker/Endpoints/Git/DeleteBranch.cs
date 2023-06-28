using Unilake.Worker.Contracts.Requests.Git;
using Unilake.Worker.Contracts.Responses.Git;
using Unilake.Worker.Events.Git;
using Unilake.Worker.Events.Git.Types;
using Unilake.Worker.Processors.PreProcessor;
using Unilake.Worker.Services;

namespace Unilake.Worker.Endpoints.Git;

public class DeleteBranch : Endpoint<GitDeleteBranchRequest, GitActionResultResponse>
{
    private readonly IProcessManager _manager;

    public DeleteBranch(IProcessManager manager)
    {
        _manager = manager;
    }

    public override void Configure()
    {
        Post("/git/branch/delete");
        Summary(s =>
        {
            s.Summary = "Deletes a branch";
            s.Description = "Deletes a branch from the local repository.";
            s.Responses[200] = "Git branch deleted queued/processed successfully.";
        });
        PreProcessors(new RequestActivityTracker<GitDeleteBranchRequest>());
    }

    public override async Task HandleAsync(GitDeleteBranchRequest request, CancellationToken cancellationToken)
    {
        string processId = _manager.GenerateProcessId(new GitActionResultResponse
        {
            Message = "Git delete branch action queued"
        });

        GitDeleteBranchTaskEvent eventDetails = request;
        eventDetails.SetProcessReferenceId(processId)
            .SetRunAsync(request.AsyncRequest)
            .SetOnCancelledMessage("Git delete branch action cancelled")
            .SetOnInProgressMessage("Git delete branch action in progress");

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