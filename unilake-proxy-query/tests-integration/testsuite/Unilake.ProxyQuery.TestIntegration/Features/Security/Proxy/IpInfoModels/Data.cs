using Unilake.ProxyQuery.TestIntegration.Features.Security.Proxy.AppInfoModels;
using Config = Unilake.ProxyQuery.TestIntegration.Shared.Config;

namespace Unilake.ProxyQuery.TestIntegration.Features.Security.Proxy.IpInfoModels;

public static class ProxyIpInfoModelTestData
{
    public static ProxyIpInfoModelDto[] GetTestData(string tenantid) => Config.GetRunScenario(tenantid) switch
    {
        "happy_flow" =>
        [
            new ProxyIpInfoModelDto
            {
                City = "London",
                Continent = "Europe",
                CountryIso2 = "GB",
                CountryName = "United Kingdom",
                IpV4 = "127.0.0.1",
                Isp = "Localhost",
                Timezone = "Europe/London"
            }
        ],
        _ => []
    };
}