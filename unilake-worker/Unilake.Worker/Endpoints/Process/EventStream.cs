using System.Runtime.CompilerServices;
using System.Threading.Channels;
using Unilake.Worker.Contracts.Requests;
using Unilake.Worker.Contracts.Responses;

namespace Unilake.Worker.Endpoints.Process;

public class EventStream : Endpoint<EventStreamRequest>
{
    private readonly ChannelReader<EventStreamResponse> _reader;
    public EventStream(ChannelReader<EventStreamResponse> reader) => _reader = reader;

    public override void Configure()
    {
        Get("/process/event-stream");
        Summary(s =>
        {
            s.Summary = "Stream results from backend";
            s.Description = "For all events that are asynchronous, this endpoint will stream the results from the backend.";
            s.Responses[200] =
                "SSE connection established successfully.";
        });
    }

    public override async Task HandleAsync(EventStreamRequest request, CancellationToken cancellationToken) => 
        await SendEventStreamAsync("worker", GetDataStream(request, cancellationToken), cancellationToken);
    
    private async IAsyncEnumerable<EventStreamResponse> GetDataStream(EventStreamRequest request, [EnumeratorCancellation] CancellationToken cancellationToken)
    {
        while (await _reader.WaitToReadAsync(cancellationToken))
        {
            var item = await _reader.ReadAsync(cancellationToken);
            if (request.Types.Contains(item.Type))
                yield return item;
        }
    }
}