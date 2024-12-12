namespace Unilake.ProxyQuery.TestIntegration.Features.Security.Proxy.AccessPoliciesModels;

public class Endpoint: Endpoint<AccessPolicyModelRequestRouteParams, AccessPolicyModelDto>
{
    public override void Configure()
    {
        AllowAnonymous();
        Get("/tenants/{tenantId}/security/proxy/access-policies/{id}");
    }

    public override async Task HandleAsync(AccessPolicyModelRequestRouteParams req, CancellationToken ct)
    {
        var found = AccessPoliciesTestData.GetTestData(req.TenantId).FirstOrDefault(x => x.PolicyId == req.Id);
        switch (found!= null)
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