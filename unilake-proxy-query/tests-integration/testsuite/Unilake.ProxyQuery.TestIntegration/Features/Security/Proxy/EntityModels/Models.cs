using Newtonsoft.Json;

namespace Unilake.ProxyQuery.TestIntegration.Features.Security.Proxy.EntityModels;

public class EntityModelRequestRouteParams : ProxyRouteParams
{
    public string Fullname { get; set; }
}

public class ProxyEntityModelDto
{

    [JsonProperty("fullName")] public string FullName { get; set; }

    [JsonProperty("id")] public string Id { get; set; }

    [JsonProperty("tags")] public string[] Tags { get; set; }

    [JsonProperty("attributes")] public List<KeyValuePair<string, string>> Attributes { get; set; }

    [JsonProperty("objects")] public Dictionary<string, ProxyObjectModelDto> Objects { get; set; }
}

public class ProxyObjectModelDto
{
    [JsonProperty("id")] public string Id { get; set; }

    [JsonProperty("fullName")] public string FullName { get; set; }
    [JsonProperty("tags")] public string[] Tags { get; set; }
    [JsonProperty("isAggregated")] public bool IsAggregated { get; set; } = false;
   [JsonProperty("name")] public string Name { get; set; }
}