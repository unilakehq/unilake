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
        await SendEventStreamAsync("update", GenerateEventsAsync(ct), ct);
    }

    private async IAsyncEnumerable<SseEventDto> GenerateEventsAsync([EnumeratorCancellation] CancellationToken ct)
    {
        while (!ct.IsCancellationRequested)
        {
            await Task.Delay(30000, ct);
            yield return new SseEventDto
            {
                TenantId = "7507f433-1943-4a7a-85e2-b8a441688709",
                InvalidationRequest = new InvalidationRequestDto
                    { CacheType = "all", Key = string.Empty }
            };
        }
    }
}