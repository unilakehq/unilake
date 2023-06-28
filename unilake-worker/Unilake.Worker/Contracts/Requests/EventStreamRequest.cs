namespace Unilake.Worker.Contracts.Requests;

// TODO: should be querystring commaseperearated?
public class EventStreamRequest
{
    [QueryParam]
    public EventStreamType[] Types { get; set; }
}

public enum EventStreamType
{
    DbtLogs,
    RequestResponse,
    IdeUpdate,
    DbtCommand,
    Ping
}