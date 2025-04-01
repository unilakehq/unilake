namespace Unilake.WebApp.DesignSystem.Services.State;

public static class StateKey
{
    /// <summary>
    /// Determines if the application is in dark mode or not.
    /// Datatype: bool
    /// </summary>
    public const string DarkMode = "DarkMode";
    /// <summary>
    /// Determines if the side navigation is collapsed or not.
    /// Datatype: bool
    /// </summary>
    public const string SideNavCollapsed = "SideNavCollapsed";
    /// <summary>
    /// Navigation menu items that are currently available.
    /// Datatype: IEnumerable[MenuItem]
    /// </summary>
    public const string NavigationMenu = "NavigationMenu";
    /// <summary>
    /// Current culture of the application.
    /// Datatype: string
    /// </summary>
    public const string Culture = "fr";
}