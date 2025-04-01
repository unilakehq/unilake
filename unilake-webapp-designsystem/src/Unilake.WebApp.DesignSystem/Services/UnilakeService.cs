using Microsoft.AspNetCore.Components;
using Microsoft.JSInterop;

namespace Unilake.WebApp.DesignSystem.Services;

public class UnilakeService(IJSRuntime jsRuntime)
{
    public async Task CopyToClipboard(string text)
    {
        await jsRuntime.InvokeVoidAsync("unilake.copyToClipboard", text);
    }

    public async Task<string> ReadFromClipboard()
    {
        return await jsRuntime.InvokeAsync<string>("unilake.readFromClipboard");
    }

    public async Task SetElementProperty(ElementReference element, string property, object value)
    {
        await jsRuntime.InvokeVoidAsync("unilake.setPropByElement", element, property, value);
    }

    public async Task DisableDraggable(ElementReference container, ElementReference element)
    {
        await jsRuntime.InvokeVoidAsync("unilake.disableDraggable", container, element);
    }
}