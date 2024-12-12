using Newtonsoft.Json;
using Unilake.ProxyQuery.TestIntegration.Shared;

namespace Unilake.ProxyQuery.TestIntegration.Features.Security.AccessPolicies.Rules;

public class AccessPolicyRuleRequestRouteParams : BaseRouteParams
{
    public string VersionId { get; set; }
}

public class AccessPolicyRuleDto
{
    [JsonProperty("eft")] public string Eft { get; set; }

    [JsonProperty("func")] public string Func { get; set; }

    [JsonProperty("object")] public string Object { get; set; }

    [JsonProperty("policy_id")] public string PolicyId { get; set; }

    [JsonProperty("policy_type")] public string PolicyType { get; set; }

    [JsonProperty("sub_rule")] public string SubRule { get; set; }
}