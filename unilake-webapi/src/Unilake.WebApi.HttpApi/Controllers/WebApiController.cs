using Unilake.WebApi.Localization;
using Volo.Abp.AspNetCore.Mvc;

namespace Unilake.WebApi.Controllers;

/* Inherit your controllers from this class.
 */
public abstract class WebApiController : AbpControllerBase
{
    protected WebApiController()
    {
        LocalizationResource = typeof(WebApiResource);
    }
}
