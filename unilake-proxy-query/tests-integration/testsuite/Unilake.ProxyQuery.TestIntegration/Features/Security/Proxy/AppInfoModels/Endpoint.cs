using Unilake.ProxyQuery.TestIntegration.Shared;

namespace Unilake.ProxyQuery.TestIntegration.Features.Security.Proxy.AppInfoModels;

public class Endpoint : Endpoint<ProxyAppInfoModelRequestDto, ProxyAppInfoModelDto>
{
    public override void Configure()
    {
        AllowAnonymous();
        Get("/tenants/{tenantId}/security/proxy/appinfo-models/{appName}");
    }

    public override async Task HandleAsync(ProxyAppInfoModelRequestDto req, CancellationToken ct)
    {
        var tenantId = Route<string>("tenantId") ?? "Unknown";
        var appName = Route<string>("appName") ?? "Unknown";

        Logger.LogInformation(
            "Requesting app info model (tenantId: {ReqTenantId}, appName: {ReqAppName}, clientProgVer: {ReqClientProgVer}, libraryName: {ReqLibraryName})",
            tenantId, appName, req.ClientProgVer, req.LibraryName);

        var found = (TestData.GetData<ProxyAppInfoModelDto>(tenantId) ?? [])
            .FirstOrDefault(x => x.AppName == appName);

        switch (found != null)
        {
            case true:
                await SendAsync(found, cancellation: ct);
                break;
            case false:
                Logger.LogWarning("AppInfoModel not found for tenant {ReqTenantId} and app name {ReqAppName}", tenantId,
                    appName);
                await SendNotFoundAsync(cancellation: ct);
                break;
        }
    }
}