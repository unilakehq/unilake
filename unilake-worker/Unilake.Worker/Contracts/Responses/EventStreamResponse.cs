using Unilake.Worker.Contracts.Requests;

namespace Unilake.Worker.Contracts.Responses;

public abstract class EventStreamResponse
{
    public EventStreamType Type { get; protected set; }
    public EventStreamDbtLogResponse DbtLog { get; protected set; }
    public EventStreamRequestResponse RequestResponse { get; protected set; }
    public EventStreamIdeUpdateResponse IdeUpdate { get; protected set; }
    public EventStreamDbtCommandResponse DbtCommandResponse { get; set; }
}
public class EventStreamDbtLogResponse : EventStreamResponse
{
    public string[] Lines { get; set; }
    public string DbtCommand { get; set; }
    public string ProcessReferenceId { get; set; }
    public new EventStreamType Type => EventStreamType.DbtLogs;

    public EventStreamDbtLogResponse(string processReferenceId, string dbtCommand, string[] lines)
    {
        ProcessReferenceId = processReferenceId;
        DbtCommand = dbtCommand;
        Lines = lines;
        DbtLog = this;
    }
}

public class EventStreamRequestResponse : EventStreamResponse
{
    public IRequestResponse Response { get; set; }
    public new EventStreamType Type => EventStreamType.RequestResponse;

    public EventStreamRequestResponse(IRequestResponse response)
    {
        Response = response;
        RequestResponse = this;
    }
}

public class EventStreamIdeUpdateResponse : EventStreamResponse
{
    public new EventStreamType Type => EventStreamType.IdeUpdate;
    public string Theme { get; set; }

    public EventStreamIdeUpdateResponse(string theme)
    {
        Theme = theme;
        IdeUpdate = this;
    }
}

public class EventStreamDbtCommandResponse : EventStreamResponse
{
    public new EventStreamType Type => EventStreamType.DbtCommand;
    public string DbtCommand { get; }
    public string ProcessReferenceId { get; }
    public string StatusMessage { get; }
    public bool? IsDone { get; }
    public bool? IsSucceeded { get; }

    public EventStreamDbtCommandResponse(string processReferenceId, string dbtCommand, string statusMessage, bool? isDone = null,
        bool? isSucceeded = null)
    {
        DbtCommand = dbtCommand;
        ProcessReferenceId = processReferenceId;
        IsDone = isDone;
        IsSucceeded = isSucceeded;
        StatusMessage = statusMessage;

        DbtCommandResponse = this;
    }
}