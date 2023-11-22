
namespace Unilake.Cli;

public static class Message
{
    public static string GreenDone => "[green]Done![/]";
    public static string TelemetryUnsupported => "[red]Telemetry is currently disabled and unsupported![/] \n\r";
    public static string CurrentState => "Current state: {0}";
    public static string ValiditionErrorsHeader => "Found the following [red]errors[/] in your configuration file: \n\r";
    public static string ValidtionErrorMessage => " :cross_mark: {0}: {1}";
    public static string ValidationConfigFileNotFound => " :cross_mark: Could not find file {0} to validate";
    public static string ValidationNoErrorsHeader => "\t:check_mark_button: No errors found in your configuration file ";
    public static string ReportUpSummaryResultStatus => "Process result: {0}  {1}";
}