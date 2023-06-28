using Flurl.Http;
using Prometheus;
using Unilake.Worker.Models.Activity;

namespace Unilake.Worker.Services.Activity;

public class ActivityReporter : BackgroundService
{
    private readonly ILogger _logger;
    private readonly IActivityTracker _tracker;
    private readonly IConfiguration _configuration;
    private Timer _timer;
    private readonly Counter _promInactiveTime = Metrics.CreateCounter(
        WorkerMetrics.ActivityReporterReportInactiveTime,
        WorkerMetrics.ActivityReporterReportInactiveTimeDesc);
    
    public ActivityReporter(IActivityTracker tracker,
        IConfiguration configuration,
        ILogger<ActivityReporter> logger)
    {
        _tracker = tracker;
        _configuration = configuration;
        _logger = logger;
    }
    
    protected override Task ExecuteAsync(CancellationToken ct)
    {
        _timer = new Timer(ReportActivity, null, TimeSpan.Zero, TimeSpan.FromSeconds(10));
        return Task.CompletedTask;
    }
    
    private void ReportActivity(object state)
    {
        if (_tracker.GetStatus().InstanceState == InstanceState.Running) return;
        _logger.LogInformation("Activity reporter is reporting inactive, notifying orchestrator");
        _promInactiveTime.IncTo(_tracker.GetStatus().LastActivityUnixTimestampUtc);
        
        var endpoint = _configuration.GetValue<string>("Environment:OrchestrationEndpoint");
        var instanceId = _configuration.GetValue<string>("Environment:OrchestrationInstanceId");
        if (string.IsNullOrEmpty(endpoint) || string.IsNullOrEmpty(instanceId))
            return;

        endpoint.PostJsonAsync(new { InstanceId = instanceId });
        
        _timer.Dispose();
    }
    
    public override Task StopAsync(CancellationToken ct)
    {
        _timer?.Dispose();
        return base.StopAsync(ct);
    }
}