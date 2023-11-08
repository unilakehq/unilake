using System.Threading.Tasks;
using Volo.Abp.DependencyInjection;

namespace Unilake.WebApi.Data;

/* This is used if database provider does't define
 * IWebApiDbSchemaMigrator implementation.
 */
public class NullWebApiDbSchemaMigrator : IWebApiDbSchemaMigrator, ITransientDependency
{
    public Task MigrateAsync()
    {
        return Task.CompletedTask;
    }
}
