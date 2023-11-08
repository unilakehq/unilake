using System.Threading.Tasks;

namespace Unilake.WebApi.Data;

public interface IWebApiDbSchemaMigrator
{
    Task MigrateAsync();
}
