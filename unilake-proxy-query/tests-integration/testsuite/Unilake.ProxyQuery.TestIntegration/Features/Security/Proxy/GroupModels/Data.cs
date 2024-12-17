using Config = Unilake.ProxyQuery.TestIntegration.Shared.Config;

namespace Unilake.ProxyQuery.TestIntegration.Features.Security.Proxy.GroupModels;

public class GroupModelsTestData
{
    public static ProxyGroupModelDto[] GetTestData(string tenantid) =>
        Config.GetRunScenario(tenantid) switch
        {
            "happy_flow" =>
            [
                new ProxyGroupModelDto
                {
                    Groups =
                    [
                        new Group
                        {
                            Id = "ecd7e759-32eb-455f-99a1-fe499fd577ef",
                            Tags = new[] { "pii::firstname", "pii::lastname" }
                        },
                        new Group
                        {
                            Id = "7d181716-a461-4a22-b5eb-64ea9c47e877",
                            Tags = new[] { "pii::firstname", "pii::lastname" }
                        }
                    ],
                    UserId = "500efbea-0bfd-49b3-88ab-090cff23cab6"
                }
            ],
            _ => []
        };
}