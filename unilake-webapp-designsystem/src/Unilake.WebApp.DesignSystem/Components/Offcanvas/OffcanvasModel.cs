using Microsoft.AspNetCore.Components;

namespace Unilake.WebApp.DesignSystem.Components;

public class OffcanvasModel
{
    internal TaskCompletionSource<OffcanvasResult> TaskSource { get; } = new();
    public Task<OffcanvasResult> Task => TaskSource.Task;
    public string Title { get; set; } = string.Empty;
    public RenderFragment? Contents { get; set; }
    public OffcanvasOptions Options { get; set; } = new();
    public string SubText { get; set; } = string.Empty;
}