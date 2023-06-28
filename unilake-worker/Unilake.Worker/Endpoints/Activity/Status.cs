using Unilake.Worker.Contracts.Responses.Activity;
using Unilake.Worker.Mappers.Activity;
using Unilake.Worker.Services.Activity;

namespace Unilake.Worker.Endpoints.Activity;

public class Status : EndpointWithoutRequest<ActivityStatusResponse, ActivityStatusResponseMapper>
{
    private readonly IActivityTracker _activityTracker;
    
    public Status(IActivityTracker tracker)
    {
        _activityTracker = tracker;
    }
    
    public override void Configure()
    {
        Get("/activity/status");
        Summary(s =>
        {
            s.Summary = "Returns the current activity status";
            s.Description = "Endpoint returns the current activity status.";
            s.Responses[200] = "Status content is returned.";
        });
    }

    public override async Task HandleAsync(CancellationToken cancellationToken)
    {
        var status = _activityTracker.GetStatus();
        await SendAsync(Map.FromEntity(status), cancellation: cancellationToken);
    }
}