using System;
using System.Collections.Generic;
using System.Text;
using Unilake.WebApi.Localization;
using Volo.Abp.Application.Services;

namespace Unilake.WebApi;

/* Inherit your application services from this class.
 */
public abstract class WebApiAppService : ApplicationService
{
    protected WebApiAppService()
    {
        LocalizationResource = typeof(WebApiResource);
    }
}
