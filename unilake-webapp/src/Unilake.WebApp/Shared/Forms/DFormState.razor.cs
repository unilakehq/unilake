using Microsoft.AspNetCore.Components;

namespace Unilake.WebApp.Shared.Forms;

/// <summary>
/// Propagates form states as a cascading parementer to child components.<br />
/// Full documentation and demos: <see href="https://havit.blazor.eu/components/HxFormState">https://havit.blazor.eu/components/HxFormState</see>
/// </summary>
public partial class DFormState
{
    /// <summary>
    /// Received form state.
    /// </summary>
    [CascadingParameter] protected FormState CascadingFormState { get; set; }

    /// <summary>
    /// Indicated enabled/disabled section. Value to propagate.
    /// </summary>
    [Parameter] public bool? Enabled { get; set; }

    /// <summary>
    /// When <c>false</c>, nothing is rendered (no children). Value is not propagated, there is no where to propagate.
    /// </summary>
    [Parameter] public bool Visible { get; set; } = true;

    /// <summary>
    /// Child content.
    /// </summary>
    [Parameter] public RenderFragment ChildContent { get; set; }

    /// <summary>
    /// Create form state to propagate.
    /// </summary>
    private FormState CreateNewCascadingFormState()
    {
        return new FormState
        {
            Enabled = Enabled ?? CascadingFormState?.Enabled,
        };
    }
}