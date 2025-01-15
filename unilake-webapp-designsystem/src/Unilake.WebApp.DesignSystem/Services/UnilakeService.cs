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
}