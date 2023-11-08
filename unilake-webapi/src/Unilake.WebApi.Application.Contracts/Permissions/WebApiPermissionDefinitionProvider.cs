using Unilake.WebApi.Localization;
using Volo.Abp.Authorization.Permissions;
using Volo.Abp.Localization;

namespace Unilake.WebApi.Permissions;

public class WebApiPermissionDefinitionProvider : PermissionDefinitionProvider
{
    public override void Define(IPermissionDefinitionContext context)
    {
        var myGroup = context.AddGroup(WebApiPermissions.GroupName);
        //Define your own permissions here. Example:
        //myGroup.AddPermission(WebApiPermissions.MyPermission1, L("Permission:MyPermission1"));
    }

    private static LocalizableString L(string name)
    {
        return LocalizableString.Create<WebApiResource>(name);
    }
}
