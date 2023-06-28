using System.Threading.Channels;
using Unilake.Worker.Contracts.Requests.Ide;
using Unilake.Worker.Contracts.Responses;
using Unilake.Worker.Processors.PreProcessor;

namespace Unilake.Worker.Endpoints.Ide;

public class UpdateSettings : Endpoint<IdeUpdateSettingsRequest>
{
    private readonly ChannelWriter<EventStreamResponse> _writer;
    
    public UpdateSettings(ChannelWriter<EventStreamResponse> writer) => _writer = writer;
    
    public override void Configure()
    {
        Post("/ide/settings");
        Summary(s =>
        {
            s.Summary = "Update IDE settings";
            s.Description = "Some settings can be influenced from outside the IDE, this endpoint is used for sending these settings.";
            s.Responses[200] =
                "Settings change request sent successfully.";
        });
        PreProcessors(new RequestActivityTracker<IdeUpdateSettingsRequest>());
    }

    public override async Task HandleAsync(IdeUpdateSettingsRequest request, CancellationToken ct)
    {
        await _writer.WriteAsync(new EventStreamIdeUpdateResponse(request.Theme), ct);
        await SendNoContentAsync(ct);
    }
}