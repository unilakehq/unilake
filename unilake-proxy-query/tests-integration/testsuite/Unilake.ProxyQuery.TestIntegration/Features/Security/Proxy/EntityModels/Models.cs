using Newtonsoft.Json;

namespace Unilake.ProxyQuery.TestIntegration.Features.Security.Proxy.EntityModels;

public class EntityModelRequestRouteParams : ProxyRouteParams
{
    public string Fullname { get; set; }
}

public class ProxyObjectModelDto
{
    [JsonProperty("entityVersion")] public long EntityVersion { get; set; }

    [JsonProperty("fullName")] public string FullName { get; set; }

    [JsonProperty("id")] public string Id { get; set; }

    [JsonProperty("lastTimeAccessed", NullValueHandling = NullValueHandling.Ignore)]
    public long? LastTimeAccessed { get; set; }

    [JsonProperty("tags")] public string[] Tags { get; set; }
}