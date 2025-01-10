using Microsoft.AspNetCore.Components;
using Microsoft.AspNetCore.Components.Web;

namespace Unilake.WebApp.DesignSystem.Components;

public abstract class UnilakeBaseComponent : ComponentBase
{
    [Parameter] public RenderFragment? ChildContent { get; set; }
    [Parameter] public EventCallback<MouseEventArgs> OnClick { get; set; }

    [Parameter(CaptureUnmatchedValues = true)]
    public IDictionary<string, object>? UnmatchedParameters { get; set; }

    protected ClassBuilder ClassBuilder => new(ProvidedCssClasses);
    private string? _providedCssClasses;

    protected string ProvidedCssClasses
    {
        get
        {
            var cssClasses = GetUnmatchedParameter("class")?.ToString();

            if (cssClasses != null)
                _providedCssClasses = cssClasses;

            return _providedCssClasses ?? string.Empty;
        }
    }

    protected virtual string ClassNames => ClassBuilder.ToString();

    protected object? GetUnmatchedParameter(string key)
    {
        if (!(UnmatchedParameters?.ContainsKey(key) ?? false)) return null;
        var value = UnmatchedParameters[key];
        UnmatchedParameters.Remove(key);
        return value;
    }
}