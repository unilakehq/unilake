using Unilake.WebApi.EntityFrameworkCore;
using Volo.Abp.Modularity;

namespace Unilake.WebApi;

[DependsOn(
    typeof(WebApiEntityFrameworkCoreTestModule)
    )]
public class WebApiDomainTestModule : AbpModule
{

}
