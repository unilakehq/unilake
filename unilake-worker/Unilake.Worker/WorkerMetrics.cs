namespace Unilake.Worker;

public class WorkerMetrics
{
    public const string HostedserviceSequentialtaskprocessorExecutionsTotal = "hostedservice_sequentialtaskprocessor_{0}_executions_total";
    public const string HostedserviceSequentialtaskprocessorExecutionsTotalDesc = "Number of tasks executed during this session.";

    public const string ActivityReporterReportCount = "activity_reporter_report_count";
    public const string ActivityReporterReportCountDesc = "Number of times the activity reporter has activity being reported.";
    
    public const string ActivityReporterReportInactiveTime = "activity_reporter_report_inactive_time";
    public const string ActivityReporterReportInactiveTimeDesc = "Timestamp of when the activity reporter has reported activity being inactive.";
    
    public const string AspnetcoreHealthcheckStatus = "aspnetcore_healthcheck_status";
    public const string AspnetcoreHealthcheckStatusDesc = "ASP.NET Core health check status (0 == Unhealthy, 1 == Healthy)";
}