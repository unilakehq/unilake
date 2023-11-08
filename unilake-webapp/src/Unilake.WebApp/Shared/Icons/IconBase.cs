namespace Unilake.WebApp.Shared.Icons;

/// <summary>
/// Base class for icons.
/// </summary>
public abstract class IconBase
{
	/// <summary>
	/// Renderer of the icon. Must have a Icon property which receives the instance of the icon (IconBase descendant instance).
	/// See BootstrapIcon and <see cref="DIcon"/> implementations for more details.
	/// </summary>
	public abstract Type RendererComponentType { get; }
}
