namespace Unilake.ProxyQuery.TestIntegration.Features.Security.AccessPolicies.Rules;

public class Endpoint : Endpoint<AccessPolicyRuleRequestRouteParams, AccessPolicyVersionDto>
{
    public override void Configure()
    {
        AllowAnonymous();
        Get("/tenants/{tenantId}/security/access-policies/rules/{versionId}");
    }

    public override async Task HandleAsync(AccessPolicyRuleRequestRouteParams req, CancellationToken ct)
    {
        if (req.VersionId == 0)
        {
            var id = 10;
            Logger.LogInformation("Version ID for Policy rules has been requested, returning: {Id}", id);
            await SendAsync(new AccessPolicyVersionDto
            {
                VersionId = id,
                AccessPolicyRules = []
            });
            return;
        }

        var found = AccessPolicyRuleTestData.GetTestData(req.TenantId);
        if (found.Length == 0)
            Logger.LogWarning("Could not find policy with ID {ReqId} for tenant {ReqTenantId}", req.VersionId,
                req.TenantId);
        else Logger.LogInformation("Returning {NumberRules} policy rules", found.Length);
        await SendAsync(new AccessPolicyVersionDto
        {
            VersionId = req.VersionId,
            AccessPolicyRules = found
        }, cancellation: ct);
    }
}