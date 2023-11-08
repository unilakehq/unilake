﻿using Microsoft.AspNetCore.Components;
using Microsoft.AspNetCore.Components.Rendering;
using Microsoft.AspNetCore.Components.Web;

namespace Unilake.WebApp.Shared;

/// <summary>
/// Renders an element with the specified name, attributes and child-content.<br />
/// </summary>
public class DDynamicElement : ComponentBase
{
	/// <summary>
	/// Gets or sets the name of the element to render.
	/// </summary>
	[Parameter] public string ElementName { get; set; } = "span";

	/// <summary>
	/// Raised after the element is clicked.
	/// </summary>
	[Parameter] public EventCallback<MouseEventArgs> OnClick { get; set; }
	/// <summary>
	/// Triggers the <see cref="OnClick"/> event. Allows interception of the event in derived components.
	/// </summary>
	protected virtual Task InvokeOnClickAsync(MouseEventArgs args) => OnClick.InvokeAsync(args);

	/// <summary>
	/// Stop onClick-event propagation. Deafult is <c>false</c>.
	/// </summary>
	[Parameter] public bool OnClickStopPropagation { get; set; }

	/// <summary>
	/// Prevents the default action for the onclick event. Deafult is <c>false</c>.
	/// </summary>
	[Parameter] public bool OnClickPreventDefault { get; set; }

	/// <summary>
	/// Element reference.
	/// </summary>
	[Parameter] public ElementReference ElementRef { get; set; }

	/// <summary>
	/// Action (synchronnous, not an EventCallback) called when the element's reference got captured.
	/// </summary>
	[Parameter] public Action<ElementReference> ElementRefChanged { get; set; }

	[Parameter] public RenderFragment ChildContent { get; set; }

	[Parameter(CaptureUnmatchedValues = true)]
	public IDictionary<string, object> AdditionalAttributes { get; set; }

	protected override void BuildRenderTree(RenderTreeBuilder builder)
	{
		builder.OpenElement(0, ElementName);

		builder.AddAttribute(1, "onclick", InvokeOnClickAsync);
		builder.AddEventPreventDefaultAttribute(2, "onclick", OnClickPreventDefault);
		builder.AddEventStopPropagationAttribute(3, "onclick", OnClickStopPropagation);
		builder.AddMultipleAttributes(4, AdditionalAttributes);
		builder.AddElementReferenceCapture(5, capturedRef =>
		{
			ElementRef = capturedRef;
			ElementRefChanged?.Invoke(ElementRef);
		});
		builder.AddContent(6, ChildContent);

		builder.CloseElement();
	}
}