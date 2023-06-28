using Unilake.Worker.Contracts.Requests.Activity;
using Unilake.Worker.Contracts.Responses.Activity;
using Unilake.Worker.Mappers.Activity;
using Unilake.Worker.Services.Activity;

namespace Unilake.Worker.Endpoints.Activity;

public class UpdateShutdownTime : Endpoint<UpdateShutdownTimeRequest, ActivityStatusResponse, ActivityStatusResponseMapper>
{
    private readonly IActivityTracker _activityTracker;
    
    public UpdateShutdownTime(IActivityTracker tracker)
    {
        _activityTracker = tracker;
    }
    
    public override void Configure()
    {
        Put("/activity/shutdown");
        Summary(s =>
        {
            s.Summary = "Adjust the current shutdown timeout";
            s.Description = "Using this endpoint you can alter the current timeout for extending or shortening a current session.";
            s.Responses[200] = "Timeout Adjusted.";
        });
    }

    public override async Task HandleAsync(UpdateShutdownTimeRequest request, CancellationToken cancellationToken)
    {
        _activityTracker.AdjustTimeout(request.AdaptTimeout);
        await SendAsync(Map.FromEntity(_activityTracker.GetStatus()), cancellation: cancellationToken);
    }
}