namespace Unilake.WebApp.Shared.Components.Tooltips;

/// <summary>
/// Placement of the tooltip for <see cref="DTooltip"/>.
/// </summary>
public enum TooltipPlacement
{
	Top = 0,
	Bottom = 1,
	Left = 2,
	Right = 3,

	/// <summary>
	/// When is specified, it will dynamically reorient the tooltip.
	/// </summary>
	Auto = 4
}
