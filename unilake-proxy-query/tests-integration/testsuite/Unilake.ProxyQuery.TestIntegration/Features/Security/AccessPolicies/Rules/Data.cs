using Config = Unilake.ProxyQuery.TestIntegration.Shared.Config;
namespace Unilake.ProxyQuery.TestIntegration.Features.Security.AccessPolicies.Rules;

public static class AccessPolicyRuleTestData
{
    public static AccessPolicyVersionRuleDto[] GetTestData(string tenantid) => Config.GetRunScenario(tenantid) switch
    {
        "happy_flow" => [
            new AccessPolicyVersionRuleDto
            {
                PolicyType = "p",
                Object = "*",
                SubRule = "true",
                Eft = "allow",
                Func = "eyJmdWxsX2FjY2VzcyI6IHRydWV9",
                PolicyId = "afe6f97a-b3b6-41b1-b4be-5c3c100e567c"
            }
        ],
        _ => []
    };
}