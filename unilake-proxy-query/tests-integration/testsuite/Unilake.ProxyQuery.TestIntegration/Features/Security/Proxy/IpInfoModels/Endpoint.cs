using Unilake.ProxyQuery.TestIntegration.Shared;

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
        var found = (TestData.GetData<ProxyIpInfoModelDto>(req.TenantId) ?? [])
            .FirstOrDefault(x => x.IpV4 == req.Ipv4);

        switch (found != null)
        {
            case true:
                await SendAsync(found, cancellation: ct);
                break;
            case false:
                Logger.LogWarning("Could not find proxy IP info model for tenant '{ReqTenantId}' and IP '{ReqIpv4}'", req.TenantId, req.Ipv4);
                await SendNotFoundAsync(cancellation: ct);
                break;
        }
    }
}