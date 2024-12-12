namespace Unilake.ProxyQuery.TestIntegration.Features.Security.Proxy.AppInfoModels;

public class Endpoint : Endpoint<ProxyAppInfoModelRequestRouteParams, ProxyAppInfoModelDto>
{
    public override void Configure()
    {
        AllowAnonymous();
        Get("/tenants/{tenantId}/security/proxy/appinfo-models/{app_name}");
    }

    public override async Task HandleAsync(ProxyAppInfoModelRequestRouteParams req, CancellationToken ct)
    {
        var found = AppInfoTestData.GetTestData(req.TenantId).FirstOrDefault(i => i.AppName == req.AppName);
        switch (found != null)
        {
            case true:
                await SendAsync(found, cancellation: ct);
                break;
            case false:
                await SendNotFoundAsync(cancellation: ct);
                break;
        }
    }
}