using Newtonsoft.Json;

namespace Unilake.ProxyQuery.TestIntegration.Features.Security.Proxy.AccessPoliciesModels;

public class AccessPolicyModelRequestRouteParams : ProxyRouteParams
{
    public string Id { get; set; }
}

public class AccessPolicyModelDto
{
    [JsonProperty("expire_datetime_utc")] public long ExpireDatetimeUtc { get; set; }

    [JsonProperty("normalized_name")] public string NormalizedName { get; set; }

    [JsonProperty("policy_id")] public string PolicyId { get; set; }

    [JsonProperty("prio_strict")] public bool PrioStrict { get; set; }
}