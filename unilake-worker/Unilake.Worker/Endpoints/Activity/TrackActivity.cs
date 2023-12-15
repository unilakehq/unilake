using Unilake.Worker.Services.Activity;

namespace Unilake.Worker.Endpoints.Activity;

public class TrackActivity : EndpointWithoutRequest
{
    private readonly IActivityTracker _activityTracker;

    public TrackActivity(IActivityTracker tracker)
    {
        _activityTracker = tracker;
    }

    public override void Configure()
    {
        Post("/activity");
        Summary(s =>
        {
            s.Summary = "Notify this worker for activity";
            s.Description = "Single post to this endpoint is enough to notify this service that there has been activity.";
            s.Responses[200] = "Activity noted.";
        });
        AuthSchemes("ApiKey", "LocalAuth");
    }

    public override async Task HandleAsync(CancellationToken ct)
        => _activityTracker.TrackActivity();
}