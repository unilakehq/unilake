using Pulumi.Automation.Events;

namespace Unilake.Cli.Stacks;

internal static class EngineEventExtensions
{
    internal static EngineEventType AsType(this EngineEvent engineEvent)
    {
        if (engineEvent.CancelEvent != null)
            return EngineEventType.CancelEvent;
        if (engineEvent.DiagnosticEvent != null)
            return EngineEventType.DiagnosticEvent;
        if (engineEvent.PolicyEvent != null)
            return EngineEventType.PolicyEvent;
        if (engineEvent.PreludeEvent != null)
            return EngineEventType.PreludeEvent;
        if (engineEvent.SummaryEvent != null)
            return EngineEventType.SummaryEvent;
        if (engineEvent.ResourceOutputsEvent != null)
            return EngineEventType.ResourceOutputsEvent;
        if (engineEvent.ResourcePreEvent != null)
            return EngineEventType.ResourcePreEvent;
        if (engineEvent.StandardOutputEvent != null)
            return EngineEventType.StandardOutputEvent;
        if (engineEvent.ResourceOperationFailedEvent != null)
            return EngineEventType.ResourceOperationFailedEvent;
        return EngineEventType.Unknown;
    }
}