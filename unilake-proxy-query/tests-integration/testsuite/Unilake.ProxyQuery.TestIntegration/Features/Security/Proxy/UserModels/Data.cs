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
                    Id = "500efbea-0bfd-49b3-88ab-090cff23cab6", AccountType = AccountType.User, EntityVersion = 1,
                    PrincipalName = "testuser@example.com", Role = "admin", Tags = ["pii::firstname", "pii::lastname"]
                }
            ],
            _ => []
        };
}