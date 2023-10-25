
namespace Unilake.Iac.Kubernetes;

public abstract class BaseInputArgs
{
    /// <summary>
    /// Specify if you want to make use of a private registry
    /// </summary>
    public bool UsePrivateRegsitry { get; set; } = false;

    /// <summary>
    /// The private regsitry base address for pulling images
    /// </summary>
    public string? PrivateRegistryBase { get; set; } = null;
}