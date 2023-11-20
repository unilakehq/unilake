using Pulumi;
using Pulumi.PostgreSql;

namespace Unilake.Iac.Kubernetes.PostgreSql;

public static class PostgreSqlProviderFactory
{
    public static Provider Create(string name, Output<string> adminUsername, Output<string> adminPassword, Output<string> databaseName, 
        Output<string> publicEndpoint, bool isSuperUser = false, ComponentResourceOptions? options = null) =>
        // create provider, to connect to the database
        new ($"{name}-postgresql", new ProviderArgs
        {
            Username = adminUsername,
            Password = adminPassword,
            Database = databaseName,
            Host = publicEndpoint,
            Superuser = isSuperUser
        }, new CustomResourceOptions
        {
            Parent = options?.Parent
        });
}