namespace Unilake.Worker.Contracts.Responses;

// TODO: also implement timeout
public enum ResultStatus
{
    Success,
    Error,
    Queued,
    InProgress,
    Cancelled
}