using Unilake.Worker.Contracts.Requests.Git;
using Unilake.Worker.Contracts.Responses.Git;
using Unilake.Worker.Events.Git;
using Unilake.Worker.Events.Git.Types;
using Unilake.Worker.Processors.PreProcessor;
using Unilake.Worker.Services;

namespace Unilake.Worker.Endpoints.Git;

public class Checkout : Endpoint<GitCheckoutRequest, GitActionResultResponse>
{
    private readonly IProcessManager _manager;

    public Checkout(IProcessManager manager)
    {
        _manager = manager;
    }

    public override void Configure()
    {
        Post("/git/checkout");
        Summary(s =>
        {
            s.Summary = "Checkout a branch or commit";
            s.Description = "Checkout a specified branch or commit in the current repository.";
            s.Responses[200] = "Git checkout action queued/processed successfully.";
        });
        PreProcessors(new RequestActivityTracker<GitCheckoutRequest>());
    }

    public override async Task HandleAsync(GitCheckoutRequest request, CancellationToken cancellationToken)
    {
        string processId = _manager.GenerateProcessId(new GitActionResultResponse
        {
            Message = "Git checkout action queued"
        });

        GitCheckoutTaskEvent eventDetails = request;
        eventDetails.SetProcessReferenceId(processId)
            .SetRunAsync(request.AsyncRequest)
            .SetOnCancelledMessage("Git checkout action cancelled")
            .SetOnInProgressMessage("Git checkout action in progress");

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