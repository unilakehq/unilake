using Newtonsoft.Json;

namespace Unilake.ProxyQuery.TestIntegration.Features.Events.sse;

public class SseEventDto
{
    [JsonProperty("tenantId")] public string TenantId { get; set; }

    [JsonProperty("invalidationRequest")] public InvalidationRequestDto? InvalidationRequest { get; set; }
}

public class InvalidationRequestDto
{
    [JsonProperty("cacheType")] public string CacheType { get; set; }

    [JsonProperty("key")] public string Key { get; set; }
}