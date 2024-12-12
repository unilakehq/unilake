namespace Unilake.ProxyQuery.TestIntegration.Features.Security.Proxy.EntityModels;

public class Endpoint : Endpoint<EntityModelRequestRouteParams, ProxyObjectModelDto>
{
    public override void Configure()
    {
        AllowAnonymous();
        Get("/tenants/{tenantId}/security/proxy/entity-models/{fullname}");
    }

    public override async Task HandleAsync(EntityModelRequestRouteParams req, CancellationToken ct)
    {
        var found = EntityModelsTestData.GetTestData(req.TenantId).FirstOrDefault(x => x.FullName == req.Fullname);
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