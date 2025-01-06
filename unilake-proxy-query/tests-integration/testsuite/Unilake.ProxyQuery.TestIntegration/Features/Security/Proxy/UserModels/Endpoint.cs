using Unilake.ProxyQuery.TestIntegration.Shared;

namespace Unilake.ProxyQuery.TestIntegration.Features.Security.Proxy.UserModels;

public class Endpoint : Endpoint<UserModelRequestRouteParams, ProxyUserModelDto>
{
    public override void Configure()
    {
        AllowAnonymous();
        Get("/tenants/{tenantId}/security/proxy/user-models/{id}");
    }

    public override async Task HandleAsync(UserModelRequestRouteParams req, CancellationToken ct)
    {
        var found = (TestData.GetData<ProxyUserModelDto>(req.TenantId) ?? [])
            .FirstOrDefault(x => x.Id == req.Id);

        switch (found != null)
        {
            case true:
                await SendAsync(found, cancellation: ct);
                break;
            case false:
                Logger.LogWarning("UserModel not found for tenant {ReqTenantId} and id {ReqId}", req.TenantId, req.Id);
                await SendNotFoundAsync(cancellation: ct);
                break;
        }
    }
}