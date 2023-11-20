using Pulumi;
using Pulumi.PostgreSql;
using Pulumi.Random;
using Provider = Pulumi.PostgreSql.Provider;

namespace Unilake.Iac.Kubernetes.PostgreSql;

public class Role : PostgreSqlComponentResource
{
    public Output<string> Username { get; }
    public Output<string?> Password { get; }

    public Role(string name, Provider provider, InputList<string> roles,
        ComponentResourceOptions? options = null) :
        base("pkg:azure:postgresql:role", name, options)
    {
        // set options
        var resourceOptions = CreateOptions(options);
        resourceOptions.Parent = this;
        
        // generate a random password for this role
        var random = new RandomPassword($"{name}-password", new RandomPasswordArgs
        {
            Length = 16,
            Special = true,
            OverrideSpecial = "@!"
        }, resourceOptions);
        var password = random.Result;

        // create role
        resourceOptions.Provider = provider;
        var role = new Pulumi.PostgreSql.Role(name, new RoleArgs
        {
            Name = name,
            Password = password,
            Login = true,
            CreateDatabase = false,
            Inherit = true,
            CreateRole = false,
            Roles = roles
        }, resourceOptions);

        // set output information
        Username = role.Name;
        Password = role.Password;
    }
}