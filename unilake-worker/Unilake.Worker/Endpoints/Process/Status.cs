using Unilake.Worker.Contracts;
using Unilake.Worker.Contracts.Requests;
using Unilake.Worker.Services;

namespace Unilake.Worker.Endpoints.Process;

public class Status : Endpoint<StatusRequest, IRequestResponse>
{
    private readonly IProcessManager _manager;
    
    public Status(IProcessManager manager)
    {
        _manager = manager;
    }
    
    public override void Configure()
    {
        Get("/process/{ProcessReferenceId}/status");
        Summary(s =>
        {
            s.Summary = "Returns the status of the specified process";
            s.Description = "After the process has been requested, the status and response will be available via this endpoint.";
            s.Responses[200] =
                "Returns the current status object.";
        });
    }

    public override async Task HandleAsync(StatusRequest request, CancellationToken cancellationToken)
    {
        await _manager.Status<IRequestResponse>(request.ProcessReferenceId).Match(
            status => SendOkAsync(status.Value, cancellation: cancellationToken),
            e =>
            {
                Logger.LogError(e.Value, "Error when returning status request");
                AddError(e.Value.Message.FirstToUpper());
                return SendErrorsAsync(cancellation: cancellationToken);
            });
    }
}