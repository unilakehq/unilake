using Config = Unilake.ProxyQuery.TestIntegration.Shared.Config;

namespace Unilake.ProxyQuery.TestIntegration.Features.Security.Proxy.AppInfoModels;

public class AppInfoTestData
{
    public static ProxyAppInfoModelDto[] GetTestData(string tenantid) => Config.GetRunScenario(tenantid) switch
    {
        "happy_flow" =>
        [
            new ProxyAppInfoModelDto
            {
                AppDriver = "Unilake.ProxyQuery.Test.Features.Security.Proxy.AppInfoModels.AppDriver",
                AppId = 1211,
                AppName = "sqlcmd",
                AppType = "Test"
            }
        ],
        _ => []
    };
}