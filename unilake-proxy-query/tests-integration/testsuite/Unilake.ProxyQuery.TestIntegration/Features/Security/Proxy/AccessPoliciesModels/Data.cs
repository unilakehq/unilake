using Config = Unilake.ProxyQuery.TestIntegration.Shared.Config;

namespace Unilake.ProxyQuery.TestIntegration.Features.Security.Proxy.AccessPoliciesModels;

public static class AccessPoliciesTestData
{
    public static AccessPolicyModelDto[] GetTestData(string tenantid) => Config.GetRunScenario(tenantid) switch
    {
        "happy_flow" => [
            new AccessPolicyModelDto
            {
                ExpireDatetimeUtc = DateTime.UtcNow.AddHours(1).GetUnixTimestamp(),
                NormalizedName = "tenant_workspace_some_policy",
                PolicyId = "afe6f97a-b3b6-41b1-b4be-5c3c100e567c",
                PrioStrict = true
            }
        ],
        _ => []
    };
}