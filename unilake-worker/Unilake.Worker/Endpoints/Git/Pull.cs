using Unilake.Worker.Contracts.Requests.Git;
using Unilake.Worker.Contracts.Responses.Git;
using Unilake.Worker.Events.Git;
using Unilake.Worker.Events.Git.Types;
using Unilake.Worker.Processors.PreProcessor;
using Unilake.Worker.Services;

namespace Unilake.Worker.Endpoints.Git;

public class Pull : Endpoint<GitPullRequest, GitActionResultResponse>
{
    private readonly IProcessManager _manager;

    public Pull(IProcessManager manager)
    {
        _manager = manager;
    }

    public override void Configure()
    {
        Post("/git/pull");
        Summary(s =>
        {
            s.Summary = "Pulls changes from a remote repository";
            s.Description = "Pulls changes from a remote repository and merges them into the current branch.";
            s.Responses[200] = "Git pull action queued/processed successfully.";
        });
        PreProcessors(new RequestActivityTracker<GitPullRequest>());
    }

    public override async Task HandleAsync(GitPullRequest request, CancellationToken cancellationToken)
    {
        string processId = _manager.GenerateProcessId(new GitActionResultResponse
        {
            Message = "Git pull action queued"
        });

        GitPullTaskEvent eventDetails = request;
        eventDetails.SetProcessReferenceId(processId)
            .SetRunAsync(request.AsyncRequest)
            .SetOnCancelledMessage("Git pull action cancelled")
            .SetOnInProgressMessage("Git pull action in progress");

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