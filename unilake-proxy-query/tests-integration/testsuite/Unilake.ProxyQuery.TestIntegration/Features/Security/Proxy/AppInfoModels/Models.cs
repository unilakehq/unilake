using Newtonsoft.Json;

namespace Unilake.ProxyQuery.TestIntegration.Features.Security.Proxy.AppInfoModels;

public class ProxyAppInfoModelRequestRouteParams : ProxyRouteParams
{
    public string AppName { get; set; }
}

public class ProxyAppInfoModelDto
{
    [JsonProperty("app_driver")] public string AppDriver { get; set; }

    [JsonProperty("app_id")] public int AppId { get; set; }

    [JsonProperty("app_name")] public string AppName { get; set; }

    [JsonProperty("app_type")] public string AppType { get; set; }
}

public class ProxyAppInfoModelRequestDto
{
    [JsonProperty("clientProgVer")] public string ClientProgVer { get; set; }

    [JsonProperty("libraryName")] public string LibraryName { get; set; }
}