namespace Unilake.WebApp.DesignSystem.Components;

public class NavItem
{
    public required string Label { get; init; }
    public string? Uri { get; init; }
    public IIcon? LeftIcon { get; init; }
    public IIcon? RightIcon { get; init; }
    public int? Badge { get; set; }
    public MenuItemState State { get; set; } = MenuItemState.Default;
    public IEnumerable<NavItem>? Children { get; init; }
    public bool IsCollapsed { get; set; }
    public bool HasChildren => Children?.Any() ?? false;
}

public class SubNavItem : NavItem
{
    public IIcon? LabelIcon { get; set; }
}

public enum MenuItemState
{
    Default,
    Active,
    Opened,
    Disabled
}