using System.Runtime.CompilerServices;

namespace Unilake.ProxyQuery.TestIntegration.Features.Events.sse;

public class Endpoint : EndpointWithoutRequest
{
    public override void Configure()
    {
        // /tenants/{tenantId}/security/proxy/access-policies/{id}
        Get("/security/proxy/event-stream");
        AllowAnonymous();
        // Options(x => x.RequireCors(p => p.AllowAnyOrigin()));
    }

    public override async Task HandleAsync(CancellationToken ct)
    {
        await SendEventStreamAsync("my-event", GenerateEventsAsync(ct), ct);
    }

    private async IAsyncEnumerable<string> GenerateEventsAsync([EnumeratorCancellation] CancellationToken ct)
    {
        while (!ct.IsCancellationRequested)
        {
            await Task.Delay(1000, ct);
            yield return $"Event: {DateTime.Now:yyyy-MM-dd HH:mm:ss}";
        }
    }
}