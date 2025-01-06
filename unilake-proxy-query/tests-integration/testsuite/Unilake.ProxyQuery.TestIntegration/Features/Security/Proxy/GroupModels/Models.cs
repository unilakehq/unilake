using Newtonsoft.Json;

namespace Unilake.ProxyQuery.TestIntegration.Features.Security.Proxy.GroupModels;

public class GroupModelRequestRouteParams : ProxyRouteParams
{
    public string Id { get; set; }
}

public class ProxyGroupModelDto
{
    [JsonProperty("groups")] public Group[] Groups { get; set; }

    [JsonProperty("userId")] public string UserId { get; set; }
}

public class Group
{
    [JsonProperty("id")] public string Id { get; set; }

    [JsonProperty("tags")] public string[] Tags { get; set; }
}