using Config = Unilake.ProxyQuery.TestIntegration.Shared.Config;

namespace Unilake.ProxyQuery.TestIntegration.Features.Security.Proxy.EntityModels;

public class EntityModelsTestData
{
    public static ProxyEntityModelDto[] GetTestData(string tenantId) => Config.GetRunScenario(tenantId) switch
    {
        "happy_flow" =>
        [
            new ProxyEntityModelDto
            {
                FullName = "default_catalog.dwh.dimaccount",
                Id = "a6881ca2-3b2a-4c94-9e78-38d9c541048f",
                Tags = ["tag1", "tag2"],
                Attributes =
                [
                    new("AccountKey", "INT"), new("ParentAccountKey", "INT"), new("AccountCodeAlternateKey", "INT"),
                    new("ParentAccountCodeAlternateKey", "INT"), new("AccountDescription", "STRING"),
                    new("AccountType", "STRING"), new("Operator", "STRING"), new("CustomMembers", "STRING"),
                    new("ValueType", "STRING"), new("CustomMemberOptions", "String")
                ],
                Objects = new Dictionary<string, ProxyObjectModelDto>
                {
                    {"default_catalog.dwh.dimaccount.AccountKey", new ProxyObjectModelDto
                    {
                        Name = "AccountKey",
                        FullName = "default_catalog.dwh.dimaccount.AcountKey",
                        Id = "some_id_I_dont_have",
                        Tags = ["custom::no_tag"],
                    }}
                }
            }
        ],
        _ => []
    };
}