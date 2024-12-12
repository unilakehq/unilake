using Unilake.ProxyQuery.TestIntegration.Features.Security.Proxy.AccessPoliciesModels;

namespace Unilake.ProxyQuery.TestIntegration.Features.Security.AccessPolicies.Rules;

public class Endpoint : Endpoint<AccessPolicyModelRequestRouteParams, AccessPolicyRuleDto[]>
{
    public override void Configure()
    {
        AllowAnonymous();
        Get("/tenants/{tenantId}/security/access-policies/rules/{versionId}");
    }

    public override async Task HandleAsync(AccessPolicyModelRequestRouteParams req, CancellationToken ct)
    {
        var found = AccessPolicyRuleTestData.GetTestData(req.TenantId);
        await SendAsync(found, cancellation: ct);
    }
}