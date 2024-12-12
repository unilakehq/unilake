using Newtonsoft.Json;

namespace Unilake.ProxyQuery.TestIntegration.Features.Security.Proxy.IpInfoModels;

public class ProxyIpInfoModelRequestRouteParams : ProxyRouteParams
{
    public string Ipv4 { get; set; }
}

public class ProxyIpInfoModelDto
{
    [JsonProperty("city")] public string City { get; set; }

    [JsonProperty("continent")] public string Continent { get; set; }

    [JsonProperty("country_iso2")] public string CountryIso2 { get; set; }

    [JsonProperty("country_name")] public string CountryName { get; set; }

    [JsonProperty("ip_v4")] public string IpV4 { get; set; }

    [JsonProperty("isp")] public string Isp { get; set; }

    [JsonProperty("timezone")] public string Timezone { get; set; }
}