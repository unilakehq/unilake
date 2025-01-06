using Newtonsoft.Json;

namespace Unilake.ProxyQuery.TestIntegration.Shared;

public static class TestData
{
    const string FileExtension = "json5";

    static string GetResourceLocation<T>(string tenantId) => typeof(T).Name switch
    {
        "AccessPolicyVersionRuleDto" => Path.Combine(Environment.CurrentDirectory, "Features", "Security",
            "AccessPolicies", "Rules", "TestData", $"{Config.GetRunScenario(tenantId)}.{FileExtension}"),
        "AccessPolicyModelDto" => Path.Combine(Environment.CurrentDirectory, "Features", "Security", "Proxy",
            "AccessPoliciesModels", "TestData", $"{Config.GetRunScenario(tenantId)}.{FileExtension}"),
        "ProxyAppInfoModelDto" => Path.Combine(Environment.CurrentDirectory, "Features", "Security", "Proxy",
            "AppInfoModels", "TestData", $"{Config.GetRunScenario(tenantId)}.{FileExtension}"),
        "ProxyEntityModelDto" => Path.Combine(Environment.CurrentDirectory, "Features", "Security", "Proxy",
            "EntityModels", "TestData", $"{Config.GetRunScenario(tenantId)}.{FileExtension}"),
        "ProxyGroupModelDto" => Path.Combine(Environment.CurrentDirectory, "Features", "Security", "Proxy",
            "GroupModels", "TestData", $"{Config.GetRunScenario(tenantId)}.{FileExtension}"),
        "ProxyIpInfoModelDto" => Path.Combine(Environment.CurrentDirectory, "Features", "Security", "Proxy",
            "IpInfoModels", "TestData", $"{Config.GetRunScenario(tenantId)}.{FileExtension}"),
        "ProxyUserModelDto" => Path.Combine(Environment.CurrentDirectory, "Features", "Security", "Proxy",
            "UserModels", "TestData", $"{Config.GetRunScenario(tenantId)}.{FileExtension}"),
        _ => throw new ArgumentOutOfRangeException(typeof(T).Name)
    };

    public static T[]? GetData<T>(string tenantId) =>
        JsonConvert.DeserializeObject<T[]>(File.ReadAllText(GetResourceLocation<T>(tenantId)));
}