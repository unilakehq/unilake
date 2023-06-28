namespace webapp.Shared.Components.Tooltips;

/// <summary>
/// Triggers for <see cref="DTooltip"/>.
/// </summary>
[Flags]
public enum TooltipTrigger
{
	Click = 1,
	Hover = 2,
	Focus = 4,
	Manual = 8
}
