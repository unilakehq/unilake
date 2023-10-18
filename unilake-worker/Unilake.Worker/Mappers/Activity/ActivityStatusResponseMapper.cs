using Unilake.Worker.Contracts.Responses.Activity;
using Unilake.Worker.Models.Activity;

namespace Unilake.Worker.Mappers.Activity;

public class ActivityStatusResponseMapper : ResponseMapper<ActivityStatusResponse, ActivityStatus>
{
    public override ActivityStatusResponse FromEntity(ActivityStatus e) => new()
    {
        InstanceState = e.InstanceState.ToString(),
        ShutdownTimeoutInSeconds = e.ShutdownTimeoutInSeconds,
        FirstActivityUnixTimestampUtc = e.FirstActivityUnixTimestampUtc,
        LastActivityUnixTimestampUtc = e.LastActivityUnixTimestampUtc,
        TimeLeftInSeconds = e.TimeLeftInSeconds
    };
}