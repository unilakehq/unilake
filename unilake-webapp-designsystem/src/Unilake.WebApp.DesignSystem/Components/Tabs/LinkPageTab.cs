using Microsoft.AspNetCore.Components;

namespace Unilake.WebApp.DesignSystem.Components;

public class LinkPageTab(Tabs parent, string labelText, int? notificationCount = null) : ITab
{
    public RenderFragment? ChildContent { get; } = null;
    public string LabelText { get; } = labelText;
    public int? NotificationCount { get; } = notificationCount;
    private Tabs Parent { get; set; } = parent;

    public void SetActiveTab()
    {
        //todo: this should be a route/navigation
    }

    public string ClassNames => "";
}