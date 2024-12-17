using Config = Unilake.ProxyQuery.TestIntegration.Shared.Config;

namespace Unilake.ProxyQuery.TestIntegration.Features.Security.Proxy.UserModels;

public class UserModelsTestData
{
    public static ProxyUserModelDto[] GetTestData(string tenantid) =>
        Config.GetRunScenario(tenantid) switch
        {
            "happy_flow" =>
            [
                new ProxyUserModelDto
                {
                    Id = "500efbea-0bfd-49b3-88ab-090cff23cab6",
                    PrincipalName = "testuser@example.com", Roles = ["admin", "data_analyst"],
                    Tags = ["pii::firstname", "pii::lastname"],
                    AccessPolicyIds = ["afe6f97a-b3b6-41b1-b4be-5c3c100e567c"]
                }
            ],
            _ => []
        };
}