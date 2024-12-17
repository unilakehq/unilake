using Newtonsoft.Json;

namespace Unilake.ProxyQuery.TestIntegration.Features.Security.Proxy.UserModels;

public class UserModelRequestRouteParams : ProxyRouteParams
{
    public string Id { get; set; }
}

/// <summary>
/// Type of account
/// </summary>
public enum AccountType
{
    Service,
    User
};

public class ProxyUserModelDto
{
    /// <summary>
    /// ID
    /// </summary>
    [JsonProperty("id")]
    public string Id { get; set; }

    [JsonProperty("principalName")] public string PrincipalName { get; set; }

    [JsonProperty("roles")] public string[] Roles { get; set; }

    [JsonProperty("tags")] public string[] Tags { get; set; }
    [JsonProperty("accessPolicyIds")] public string[] AccessPolicyIds { get; set; }
}