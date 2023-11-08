using Unilake.WebApi.EntityFrameworkCore;
using Volo.Abp.Autofac;
using Volo.Abp.Modularity;

namespace Unilake.WebApi.DbMigrator;

[DependsOn(
    typeof(AbpAutofacModule),
    typeof(WebApiEntityFrameworkCoreModule),
    typeof(WebApiApplicationContractsModule)
    )]
public class WebApiDbMigratorModule : AbpModule
{
}
