using Newtonsoft.Json;
using Unilake.ProxyQuery.TestIntegration.Shared;

namespace Unilake.ProxyQuery.TestIntegration.Features.Security.AccessPolicies.Rules;

public class AccessPolicyRuleRequestRouteParams : BaseRouteParams
{
    public long VersionId { get; set; }
}

public class AccessPolicyVersionDto
{
    [JsonProperty("versionId")] public long VersionId { get; set; }
    [JsonProperty("rules")] public AccessPolicyVersionRuleDto[] AccessPolicyRules { get; set; }
}

public class AccessPolicyVersionRuleDto
{
    [JsonProperty("e")] public string Eft { get; set; }

    [JsonProperty("f")] public string Func { get; set; }

    [JsonProperty("o")] public string Object { get; set; }

    [JsonProperty("i")] public string PolicyId { get; set; }

    [JsonProperty("t")] public string PolicyType { get; set; }

    [JsonProperty("s")] public string SubRule { get; set; }
}