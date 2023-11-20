using Pulumi;

namespace Unilake.Iac.Kubernetes.PostgreSql;

public abstract class PostgreSqlComponentResource : InternalComponentResource<NamingConventionPostgreSql>
{
    protected PostgreSqlComponentResource(string type, string name, ComponentResourceOptions? options = null, bool checkNamingConvention = true) : base(
        type, name, options, checkNamingConvention)
    {
    }

    protected PostgreSqlComponentResource(string type, string name, ResourceArgs? args,
        ComponentResourceOptions? options = null, bool remote = false) : base(type, name, args, options, remote)
    {
        throw new NotImplementedException();
    }
}