namespace Unilake.WebApp.Components;

public record ProcessStatusItem(string Title, string Description, ProcessStatusIndicatorType StatusIndicator);

public enum ProcessStatusIndicatorType
{
    Succeeded,
    Failed,
    Running,
    Unknown
}
