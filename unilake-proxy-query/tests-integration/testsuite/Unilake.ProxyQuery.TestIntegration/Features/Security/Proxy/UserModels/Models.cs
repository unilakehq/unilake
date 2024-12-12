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
    /// Type of account
    /// </summary>
    [JsonProperty("accountType")]
    public AccountType AccountType { get; set; }

    [JsonProperty("entityVersion")] public long EntityVersion { get; set; }

    /// <summary>
    /// ID
    /// </summary>
    [JsonProperty("id")]
    public string Id { get; set; }

    [JsonProperty("principalName")] public string PrincipalName { get; set; }

    [JsonProperty("role")] public string Role { get; set; }

    [JsonProperty("tags")] public string[] Tags { get; set; }
}