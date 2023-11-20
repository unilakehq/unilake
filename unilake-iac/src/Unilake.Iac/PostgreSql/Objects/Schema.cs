using Pulumi;
using Pulumi.PostgreSql;

namespace Unilake.Iac.Kubernetes.PostgreSql;

public class Schema : PostgreSqlComponentResource
{
    public Output<string> Name { get; }

    public Schema(string name, Provider provider, Output<string> databaseName, string schemaName, string owner,
        bool dropCascade = false, bool ifNotExists = true, ComponentResourceOptions? options = null)
        : base("pkg:azure:postgresql:schema", name, options)
    {
        // set options
        var resourceOptions = CreateOptions(options);
        resourceOptions.Parent = this;

        // create resources
        resourceOptions.Provider = provider;
        var schema = new Pulumi.PostgreSql.Schema(name, new SchemaArgs
        {
            Name = schemaName,
            Database = databaseName,
            Owner = owner,
            DropCascade = dropCascade,
            IfNotExists = ifNotExists
        }, resourceOptions);

        // set properties
        Name = schema.Name;
    }
}