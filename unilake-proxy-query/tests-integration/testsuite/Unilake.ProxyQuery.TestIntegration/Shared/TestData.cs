using Newtonsoft.Json;

namespace Unilake.ProxyQuery.TestIntegration.Shared;

public static class TestData
{
    static string GetResourceLocation<T>(string tenantId) => typeof(T).Name switch
    {
        "AccessPolicyVersionRuleDto" => Path.Combine(Environment.CurrentDirectory, "Features", "Security",
            "AccessPolicies", "Rules", "TestData", $"{Config.GetRunScenario(tenantId)}.json"),
        "AccessPolicyModelDto" => Path.Combine(Environment.CurrentDirectory, "Features", "Security", "Proxy",
            "AccessPoliciesModels", "TestData", $"{Config.GetRunScenario(tenantId)}.json"),
        "ProxyAppInfoModelDto" => Path.Combine(Environment.CurrentDirectory, "Features", "Security", "Proxy",
            "AppInfoModels", "TestData", $"{Config.GetRunScenario(tenantId)}.json"),
        "ProxyEntityModelDto" => Path.Combine(Environment.CurrentDirectory, "Features", "Security", "Proxy",
            "EntityModels", "TestData", $"{Config.GetRunScenario(tenantId)}.json"),
        "ProxyGroupModelDto" => Path.Combine(Environment.CurrentDirectory, "Features", "Security", "Proxy",
            "GroupModels", "TestData", $"{Config.GetRunScenario(tenantId)}.json"),
        "ProxyIpInfoModelDto" => Path.Combine(Environment.CurrentDirectory, "Features", "Security", "Proxy",
            "IpInfoModels", "TestData", $"{Config.GetRunScenario(tenantId)}.json"),
        "ProxyUserModelDto" => Path.Combine(Environment.CurrentDirectory, "Features", "Security", "Proxy",
            "UserModels", "TestData", $"{Config.GetRunScenario(tenantId)}.json"),
        _ => throw new ArgumentOutOfRangeException(typeof(T).Name)
    };

    public static T[]? GetData<T>(string tenantId) =>
        JsonConvert.DeserializeObject<T[]>(File.ReadAllText(GetResourceLocation<T>(tenantId)));
}