using Config = Unilake.ProxyQuery.TestIntegration.Shared.Config;
namespace Unilake.ProxyQuery.TestIntegration.Features.Security.Proxy.EntityModels;

public class EntityModelsTestData
{
    public static ProxyObjectModelDto[] GetTestData(string tenantId) => Config.GetRunScenario(tenantId) switch
    {
        "happy_flow" => [
            new ProxyObjectModelDto
            {
                EntityVersion = 1,
                FullName = "some_catalog.dwh.DimCustomer",
                Id = "a6881ca2-3b2a-4c94-9e78-38d9c541048f",
                LastTimeAccessed = DateTime.UtcNow.GetUnixTimestamp(),
                Tags = new[] { "tag1", "tag2" }
            }
        ],
        _ => []
    };
}