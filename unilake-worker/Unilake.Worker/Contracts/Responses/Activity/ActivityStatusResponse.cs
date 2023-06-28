namespace Unilake.Worker.Contracts.Responses.Activity;

public class ActivityStatusResponse
{
    public long FirstActivityUnixTimestampUtc { get; set; }
    public long LastActivityUnixTimestampUtc { get; set; }
    public long ShutdownTimeoutInSeconds { get; set; }
    public long TimeLeftInSeconds { get; set; }
    public string InstanceState { get; set; }
}