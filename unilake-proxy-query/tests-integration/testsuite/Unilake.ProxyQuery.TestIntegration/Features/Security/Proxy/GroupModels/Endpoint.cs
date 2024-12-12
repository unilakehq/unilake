namespace Unilake.ProxyQuery.TestIntegration.Features.Security.Proxy.GroupModels;

public class Endpoint : Endpoint<GroupModelRequestRouteParams, ProxyGroupModelDto>
{
    public override void Configure()
    {
        AllowAnonymous();
        Get("/tenants/{tenantId}/security/proxy/group-models/{id}");
    }

    public override async Task HandleAsync(GroupModelRequestRouteParams req, CancellationToken ct)
    {
        var found = GroupModelsTestData.GetTestData(req.TenantId).FirstOrDefault(g => g.UserId == req.Id);
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