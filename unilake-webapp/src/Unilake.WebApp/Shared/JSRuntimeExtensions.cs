using System.Reflection;
using Havit.Diagnostics.Contracts;
using Microsoft.JSInterop;

namespace Unilake.WebApp.Shared;
public static class JSRuntimeExtensions
{
	internal static ValueTask<IJSObjectReference> ImportHavitBlazorBootstrapModuleAsync(this IJSRuntime jsRuntime, string moduleNameWithoutExtension)
	{
		versionIdentifierHavitBlazorBootstrap ??= GetAssemblyVersionIdentifierForUri(typeof(ThemeColor).Assembly);

		var path = "./_content/Havit.Blazor.Components.Web.Bootstrap/" + moduleNameWithoutExtension + ".js?v=" + versionIdentifierHavitBlazorBootstrap;
		return jsRuntime.InvokeAsync<IJSObjectReference>("import", path);
	}
	
	private static string versionIdentifierHavitBlazorBootstrap;
	
	public static ValueTask<IJSObjectReference> ImportModuleAsync(this IJSRuntime jsRuntime, string modulePath, Assembly assemblyForVersionInfo = null)
	{
		Contract.Requires<ArgumentException>(!String.IsNullOrWhiteSpace(modulePath));

		if (assemblyForVersionInfo is not null)
		{
			modulePath = modulePath + "?v=" + GetAssemblyVersionIdentifierForUri(assemblyForVersionInfo);
		}
		return jsRuntime.InvokeAsync<IJSObjectReference>("import", modulePath);
	}

	internal static ValueTask<IJSObjectReference> ImportHavitBlazorWebModuleAsync(this IJSRuntime jsRuntime, string moduleNameWithoutExtension)
	{
		versionIdentifierHavitBlazorWeb ??= GetAssemblyVersionIdentifierForUri(typeof(DDynamicElement).Assembly);

		var path = "./_content/Havit.Blazor.Components.Web/" + moduleNameWithoutExtension + ".js?v=" + versionIdentifierHavitBlazorWeb;
		return jsRuntime.InvokeAsync<IJSObjectReference>("import", path);
	}
	private static string versionIdentifierHavitBlazorWeb;

	internal static string GetAssemblyVersionIdentifierForUri(Assembly assembly)
	{
		return Uri.EscapeDataString(((AssemblyInformationalVersionAttribute)Attribute.GetCustomAttribute(assembly, typeof(AssemblyInformationalVersionAttribute), false)).InformationalVersion);
	}
}