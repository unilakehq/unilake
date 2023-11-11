using Pulumi;

namespace Unilake.Iac;

public abstract class InternalComponentResource<T> : ComponentResource
    where T : NamingConvention, new()
{
    public static T NamingConvention { get; } = new();

    protected InternalComponentResource(string type, string name, ComponentResourceOptions? options = null, bool checkNamingConvention = true) : base(type,
        name, options)
    {
        // See if we need to check the naming convention
        if (!checkNamingConvention)
            return;

        // Check naming convention
        var check = NamingConvention.IsCompliant(name, type);
        if (!check.isSuccess)
            throw new ArgumentException($"{name} is not a valid naming convention. Error message: {check.errorMessage}",
                nameof(name));
    }

    protected InternalComponentResource(string type, string name, ResourceArgs? args,
        ComponentResourceOptions? options = null, bool remote = false) : base(type, name, args, options, remote)
    {
    }

    protected CustomResourceOptions CreateOptions(ComponentResourceOptions? options = null, Resource? parent = null)
    {
        return new()
        {
            DependsOn = options?.DependsOn ?? new InputList<Resource>(),
            Parent = parent ?? options?.Parent
        };
    }

    protected ComponentResourceOptions CopyOptions(ComponentResourceOptions? options = null, Resource? parent = null)
    {
        return options == null
            ? new ComponentResourceOptions
            {
                Parent = parent
            }
            : new ComponentResourceOptions
            {
                Aliases = options.Aliases,
                Id = options.Id,
                Parent = parent ?? options.Parent,
                Protect = options.Protect,
                Provider = options.Provider,
                Providers = options.Providers,
                Urn = options.Urn,
                Version = options.Version,
                CustomTimeouts = options.CustomTimeouts,
                DependsOn = options.DependsOn,
                IgnoreChanges = options.IgnoreChanges,
                ResourceTransformations = options.ResourceTransformations,
                ReplaceOnChanges = options.ReplaceOnChanges,
                RetainOnDelete = options.RetainOnDelete,
                PluginDownloadURL = options.PluginDownloadURL
            };
    }
}