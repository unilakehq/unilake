using Pulumi;

namespace Unilake.Iac.Kubernetes.PostgreSql;

public class Database : PostgreSqlComponentResource
{
    public Database(string name, ComponentResourceOptions? options = null, bool checkNamingConvention = true) 
        : base("", name, options, checkNamingConvention)
    {
    }
}