namespace Unilake.Worker.Models.Activity;

public class ActivityStatus
{
    public long FirstActivityUnixTimestampUtc { get; set; }
    public long LastActivityUnixTimestampUtc { get; set; }
    public long ShutdownTimeoutInSeconds { get; set; }
    public long TimeLeftInSeconds { get; set; }
    public InstanceState InstanceState { get; set; }
}