namespace Unilake.ProxyQuery.TestIntegration.Features.Security.Proxy.IpInfoModels;

public class Endpoint : Endpoint<ProxyIpInfoModelRequestRouteParams, ProxyIpInfoModelDto>
{
    public override void Configure()
    {
        AllowAnonymous();
        Get("/tenants/{tenantId}/security/proxy/ipinfo-models/{ipv4}");
    }

    public override async Task HandleAsync(ProxyIpInfoModelRequestRouteParams req, CancellationToken ct)
    {
        var found = ProxyIpInfoModelTestData.GetTestData(req.TenantId).FirstOrDefault(x => x.IpV4 == req.Ipv4);
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