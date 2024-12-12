namespace Unilake.ProxyQuery.TestIntegration.Shared;

public static class Config
{
    public static string GetRunScenario(string tenantid) => new[] { ("happy_flow", "7507f433-1943-4a7a-85e2-b8a441688709"), ("some_other_scenario", "todo") }.FirstOrDefault(x => x.Item2 == tenantid).Item1;
}