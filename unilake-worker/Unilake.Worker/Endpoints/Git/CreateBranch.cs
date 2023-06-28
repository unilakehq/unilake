using Unilake.Worker.Contracts.Requests.Git;
using Unilake.Worker.Contracts.Responses.Git;
using Unilake.Worker.Events.Git;
using Unilake.Worker.Events.Git.Types;
using Unilake.Worker.Processors.PreProcessor;
using Unilake.Worker.Services;

namespace Unilake.Worker.Endpoints.Git;

public class CreateBranch : Endpoint<GitCreateBranchRequest, GitActionResultResponse>
{
    private readonly IProcessManager _manager;

    public CreateBranch(IProcessManager manager)
    {
        _manager = manager;
    }

    public override void Configure()
    {
        Post("/git/branch/create");
        Summary(s =>
        {
            s.Summary = "Creates a new branch";
            s.Description = "Creates a new branch in the local repository.";
            s.Responses[200] = "Git branch created queued/processed successfully.";
        });
        PreProcessors(new RequestActivityTracker<GitCreateBranchRequest>());
    }

    public override async Task HandleAsync(GitCreateBranchRequest request, CancellationToken cancellationToken)
    {
        string processId = _manager.GenerateProcessId(new GitActionResultResponse
        {
            Message = "Git create branch action queued"
        });

        GitCreateBranchTaskEvent eventDetails = request;
        eventDetails.SetProcessReferenceId(processId)
            .SetRunAsync(request.AsyncRequest)
            .SetOnCancelledMessage("Git create branch action cancelled")
            .SetOnInProgressMessage("Git create branch action in progress");

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
