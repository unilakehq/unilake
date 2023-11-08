using Microsoft.AspNetCore.Components;
using Unilake.WebApp.Shared.Components.Tooltips.Internal;

namespace Unilake.WebApp.Shared.Components.Tooltips;

/// <summary>
/// <see href="https://getbootstrap.com/docs/5.2/components/tooltips/">Bootstrap Tooltip</see> component, activates on hover.<br />
/// Rendered as a <c>span</c> wrapper to fully support tooltips on disabled elements (see example in <see href="https://getbootstrap.com/docs/5.2/components/tooltips/#disabled-elements">Disabled elements</see> in the Bootstrap tooltip documentation).<br />
/// </summary>
public class DTooltip : DTooltipInternalBase
{
	/// <summary>
	/// Tooltip text.
	/// </summary>
	[Parameter]
	public string Text
	{
		get => TitleInternal;
		set => TitleInternal = value;
	}

	/// <summary>
	/// Tooltip placement. Default is <see cref="TooltipPlacement.Top"/>.
	/// </summary>
	[Parameter]
	public TooltipPlacement Placement
	{
		get => PlacementInternal;
		set => PlacementInternal = value;
	}

	/// <summary>
	/// Tooltip trigger(s). Default is <c><see cref="TooltipTrigger.Hover"/> | <see cref="TooltipTrigger.Focus"/></c>.
	/// </summary>
	[Parameter]
	public TooltipTrigger Trigger
	{
		get => TriggerInternal;
		set => TriggerInternal = value;
	}


	protected override string JsModuleName => nameof(DTooltip);
	protected override string DataBsToggle => "tooltip";

	public DTooltip()
	{
		Placement = TooltipPlacement.Top;
		Trigger = TooltipTrigger.Hover | TooltipTrigger.Focus;
	}
}
