using Unilake.Worker.Contracts;
using Unilake.Worker.Contracts.Requests;
using Unilake.Worker.Services;

namespace Unilake.Worker.Endpoints.Process;

public class Cancel : Endpoint<CancelRequest, IRequestResponse>
{
    private readonly IProcessManager _manager;
    
    public Cancel(IProcessManager manager)
    {
        _manager = manager;
    }
    
    public override void Configure()
    {
        Get("/process/{ProcessReferenceId}/cancel");
        Summary(s =>
        {
            s.Summary = "Cancels a queued process";
            s.Description = "If needed, an already queued process can be cancelled.";
            s.Responses[200] =
                "If cancellation is successful, the process will be removed from the queue.";
        });
    }
    
    public override async Task HandleAsync(CancelRequest request, CancellationToken cancellationToken)
    {
        await _manager.Cancel(request.ProcessReferenceId).Match(
            status => SendOkAsync(status.Value, cancellation: cancellationToken),
            e =>
            {
                Logger.LogError(e.Value, "Error when processing cancellation request");
                AddError(e.Value.Message.FirstToUpper());
                return SendErrorsAsync(cancellation: cancellationToken);
            });
    }
}