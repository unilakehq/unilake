using Unilake.Worker.Contracts.Requests.Git;
using Unilake.Worker.Contracts.Responses.Git;
using Unilake.Worker.Events.Git;
using Unilake.Worker.Events.Git.Types;
using Unilake.Worker.Processors.PreProcessor;
using Unilake.Worker.Services;

namespace Unilake.Worker.Endpoints.Git;

public class Fetch : Endpoint<GitFetchRequest, GitActionResultResponse>
{
    private readonly IProcessManager _manager;

    public Fetch(IProcessManager manager)
    {
        _manager = manager;
    }

    public override void Configure()
    {
        Post("/git/fetch");
        Summary(s =>
        {
            s.Summary = "Fetches changes from a remote repository";
            s.Description = "Fetches changes from a remote repository without merging them.";
            s.Responses[200] = "Git fetch action queued/processed successfully.";
        });
        PreProcessors(new RequestActivityTracker<GitFetchRequest>());
    }

    public override async Task HandleAsync(GitFetchRequest request, CancellationToken cancellationToken)
    {
        string processId = _manager.GenerateProcessId(new GitActionResultResponse
        {
            Message = "Git fetch action queued"
        });

        GitFetchTaskEvent eventDetails = request;
        eventDetails.SetProcessReferenceId(processId)
            .SetRunAsync(request.AsyncRequest)
            .SetOnCancelledMessage("Git fetch action cancelled")
            .SetOnInProgressMessage("Git fetch action in progress");

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