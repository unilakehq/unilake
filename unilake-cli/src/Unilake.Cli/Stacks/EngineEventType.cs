namespace Unilake.Cli.Stacks;

internal enum EngineEventType
{
    DiagnosticEvent,
    CancelEvent,
    Unknown,
    PolicyEvent,
    PreludeEvent,
    SummaryEvent,
    ResourceOutputsEvent,
    ResourcePreEvent,
    StandardOutputEvent,
    ResourceOperationFailedEvent
}