using Microsoft.AspNetCore.Components;
using Unilake.WebApp.DesignSystem.Components;

namespace Unilake.WebApp.DesignSystem.Services;

public class OffcanvasService : IOffcanvasService
{
    public event Action? OnChanged;
    private readonly Stack<OffcanvasModel> _models = new();
    public IEnumerable<OffcanvasModel> Models => _models;

    public Task<OffcanvasResult> ShowAsync<TComponent>(string title, RenderComponent<TComponent> component,
        OffcanvasOptions? options = null) where TComponent : IComponent
    {
        var offcanvasModel = new OffcanvasModel
        {
            Title = title,
            Contents = component.Contents,
            Options = options ?? new OffcanvasOptions()
        };
        _models.Push(offcanvasModel);
        OnChanged?.Invoke();
        return offcanvasModel.Task;
    }

    public void Close()
    {
        if (_models.Count != 0)
            _models.Pop();
        OnChanged?.Invoke();
    }
}