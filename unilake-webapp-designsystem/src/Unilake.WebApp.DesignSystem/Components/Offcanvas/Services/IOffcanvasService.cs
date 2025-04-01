using Microsoft.AspNetCore.Components;

namespace Unilake.WebApp.DesignSystem.Components.Offcanvas.Services;

public interface IOffcanvasService
{
    IEnumerable<OffcanvasModel> Models { get; }
    event Action OnChanged;
    void Close();
    Task<OffcanvasResult> ShowAsync<TComponent>(string title, RenderComponent<TComponent> component,
        OffcanvasOptions? options = null) where TComponent : IComponent;
}