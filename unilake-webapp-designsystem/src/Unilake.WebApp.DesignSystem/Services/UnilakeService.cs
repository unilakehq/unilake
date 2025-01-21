using Microsoft.AspNetCore.Components;
using Microsoft.JSInterop;

namespace Unilake.WebApp.DesignSystem.Services;

public class UnilakeService
{
    private readonly IJSRuntime _jsRuntime;

    public UnilakeService(IJSRuntime jsRuntime)
    {
        _jsRuntime = jsRuntime;
    }

    public async Task CopyToClipboard(string text)
    {
        await _jsRuntime.InvokeVoidAsync("unilake.copyToClipboard", text);
    }

    public async Task<string> ReadFromClipboard()
    {
        return await _jsRuntime.InvokeAsync<string>("unilake.readFromClipboard");
    }

    public async Task SetElementProperty(ElementReference element, string property, object value)
    {
        await _jsRuntime.InvokeVoidAsync("unilake.setPropByElement", element, property, value);
    }

    public async Task DisableDraggable(ElementReference container, ElementReference element)
    {
        await _jsRuntime.InvokeVoidAsync("unilake.disableDraggable", container, element);
    }
}