namespace Unilake.WebApp.Shared.Components.Tooltips;

/// <summary>
/// Triggers for <see cref="DPopover"/>.
/// </summary>
[Flags]
public enum PopoverTrigger
{
	Click = TooltipTrigger.Click,
	Hover = TooltipTrigger.Hover,
	Focus = TooltipTrigger.Focus,
	Manual = TooltipTrigger.Manual
}
