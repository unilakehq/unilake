using Unilake.ProxyQuery.TestIntegration.Shared;

namespace Unilake.ProxyQuery.TestIntegration.Features.Security.Proxy.EntityModels;

public class Endpoint : Endpoint<EntityModelRequestRouteParams, ProxyEntityModelDto>
{
    public override void Configure()
    {
        AllowAnonymous();
        Get("/tenants/{tenantId}/security/proxy/entity-models/{fullname}");
    }

    public override async Task HandleAsync(EntityModelRequestRouteParams req, CancellationToken ct)
    {
        var found = (TestData.GetData<ProxyEntityModelDto>(req.TenantId) ?? [])
            .FirstOrDefault(x => x.FullName == req.Fullname);

        switch (found != null)
        {
            case true:
                await SendAsync(found, cancellation: ct);
                break;
            case false:
                Logger.LogWarning("Entity model not found for tenant {TenantId} and fullname {Fullname}", req.TenantId,
                    req.Fullname);
                await SendNotFoundAsync(cancellation: ct);
                break;
        }
    }
}