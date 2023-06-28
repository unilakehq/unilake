using Prometheus;
using Unilake.Worker.Models.Activity;

namespace Unilake.Worker.Services.Activity;

public interface IActivityTracker
{
    void TrackActivity();
    ActivityStatus GetStatus();
    void AdjustTimeout(long adjustment);
}

public class ActivityTracker : IActivityTracker
{
    private long _firstActivity;
    private long _lastActivity;
    private InstanceState _state = InstanceState.Running;
    private readonly ILogger _logger;
    private readonly long _shutdownTimeoutInSeconds;
    private long _currentShutdownTime;
    
    private readonly Counter _promExecutionCount = Metrics.CreateCounter(
        WorkerMetrics.ActivityReporterReportCount,
        WorkerMetrics.ActivityReporterReportCountDesc);

    public ActivityTracker(IConfiguration configuration, ILogger<ActivityTracker> logger)
    {
        _logger = logger;
        _shutdownTimeoutInSeconds = configuration.GetValue<long>("Environment:ShutdownTimeoutInSeconds");
        _currentShutdownTime = configuration.GetValue<long>("Environment:ShutdownTimePeriodInSeconds");
    }
    
    public void TrackActivity()
    {
        var currentTime = GetCurrentUnixTimestampUtc();
        if (_firstActivity == 0)
        {
            _firstActivity = currentTime;
            _currentShutdownTime += currentTime;
        }

        if (_state == InstanceState.PendingShutdown || TimeLeft(currentTime) == 0)
            return;

        _lastActivity = currentTime;
        _logger.LogInformation("Tracked activity at {LastActivity}", _lastActivity);
        _promExecutionCount.Inc();
    }
    
    private long TimeLeft(long currentTime) => 
        Math.Max(_currentShutdownTime > currentTime ? _currentShutdownTime - currentTime : _shutdownTimeoutInSeconds - (currentTime-_lastActivity), 0); 

    public ActivityStatus GetStatus()
    {
        var currentTime = GetCurrentUnixTimestampUtc();
        var timeElapsed = currentTime - _lastActivity;

        var timeLeft = TimeLeft(currentTime);
        ChangeState(_firstActivity > 0 && 
                    timeElapsed > _shutdownTimeoutInSeconds && 
                    currentTime >= _currentShutdownTime ? InstanceState.PendingShutdown : InstanceState.Running);
        var status = new ActivityStatus
        {
            FirstActivityUnixTimestampUtc = _firstActivity,
            LastActivityUnixTimestampUtc= _lastActivity,
            ShutdownTimeoutInSeconds = _shutdownTimeoutInSeconds,
            TimeLeftInSeconds = timeLeft,
            InstanceState = _state
        };
        
        _logger.LogDebug("Current ActivityStatus: FirstActivityUnixTimestampUtc={FirstActivity}, " +
                               "LastActivityUnixTimestampUtc={LastActivity}, " +
                               "ShutdownTimeoutInSeconds={ShutdownTimeout}, " +
                               "TimeLeftInSeconds={TimeLeft}, " +
                               "InstanceState={InstanceState}", 
                                status.FirstActivityUnixTimestampUtc, 
                                status.LastActivityUnixTimestampUtc, 
                                status.ShutdownTimeoutInSeconds, 
                                status.TimeLeftInSeconds, 
                                status.InstanceState);
        return status;
    }

    public void AdjustTimeout(long adjustment)
    {
        _currentShutdownTime += adjustment;
        TrackActivity();
    }

    private void ChangeState(InstanceState state)
    {
        if (state != _state)
            _logger.LogInformation("Changing current state from {OldInstanceState} to {NewInstanceState}", _state, state);
        _state = state;
    }
    
    protected virtual long GetCurrentUnixTimestampUtc()
    {
        var unixEpochTime = new DateTime(1970, 1, 1, 0, 0, 0, DateTimeKind.Utc);
        var currentTime = DateTime.UtcNow;
        var unixTimestamp = (long)(currentTime - unixEpochTime).TotalSeconds;
        return unixTimestamp;
    }
}