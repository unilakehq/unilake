using Unilake.Worker.Contracts.Requests.Dbt;
using Unilake.Worker.Contracts.Responses.Dbt;
using Unilake.Worker.Events.Dbt;
using Unilake.Worker.Events.Dbt.Types;
using Unilake.Worker.Processors.PreProcessor;
using Unilake.Worker.Services;

namespace Unilake.Worker.Endpoints.Dbt;

public class BuildModel : Endpoint<BuildModelRequest, DbtActionResultResponse>
{
    private readonly IProcessManager _manager;
    
    public BuildModel(IProcessManager manager)
    {
        _manager = manager;
    }

    public override void Configure()
    {
        Post("");
        Summary(s =>
        {
            s.Summary = "Build a dbt model";
            s.Description = "";
            s.Responses[200] = "";
        });
        PreProcessors(new RequestActivityTracker<BuildModelRequest>());
    }
    
    public override async Task HandleAsync(BuildModelRequest request, CancellationToken cancellationToken)
    {
        string processId = _manager.GenerateProcessId(new DbtActionResultResponse
        {
            Message = "Dbt build model action queued"
        });

        BuildModelTaskEvent eventDetails = new BuildModelTaskEvent();
        eventDetails.SetProcessReferenceId(processId)
            .SetRunAsync(request.AsyncRequest)
            .SetOnCancelledMessage("Dbt build model action cancelled")
            .SetOnInProgressMessage("Dbt build model action in progress");

        await _manager.PublishEventAsync<DbtTaskEvent>(eventDetails, request.GetMode(), cancellationToken).ConfigureAwait(false);
        await _manager.Status<DbtActionResultResponse>(processId).Match(
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