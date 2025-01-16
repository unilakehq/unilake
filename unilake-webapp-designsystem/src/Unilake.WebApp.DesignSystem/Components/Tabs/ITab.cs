using Microsoft.AspNetCore.Components;

namespace Unilake.WebApp.DesignSystem.Components;

public interface ITab
{
    RenderFragment? ChildContent { get; }

    string LabelText { get; }

    int? NotificationCount { get; }

    void SetActiveTab();
}