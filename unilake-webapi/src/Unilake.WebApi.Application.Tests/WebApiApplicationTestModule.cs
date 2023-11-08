using Volo.Abp.Modularity;

namespace Unilake.WebApi;

[DependsOn(
    typeof(WebApiApplicationModule),
    typeof(WebApiDomainTestModule)
    )]
public class WebApiApplicationTestModule : AbpModule
{

}
