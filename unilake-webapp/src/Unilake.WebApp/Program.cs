using Microsoft.AspNetCore.Components.Web;
using Microsoft.AspNetCore.Components.WebAssembly.Hosting;
using Unilake.WebApp;
using Unilake.WebApp.DesignSystem;
using Unilake.WebApp.DesignSystem.Services.Messaging;
using Unilake.WebApp.DesignSystem.Services.State;

var builder = WebAssemblyHostBuilder.CreateDefault(args);
builder.RootComponents.Add<App>("#app");
builder.RootComponents.Add<HeadOutlet>("head::after");

builder.Services.AddScoped(_ => new HttpClient { BaseAddress = new Uri(builder.HostEnvironment.BaseAddress) });
builder.Services.AddScoped<StateService, StaticStateServiceImpl>();
builder.Services.AddScoped<MessageService, SseMessageServiceImpl>();
builder.Services.AddUnilakeDesignSystem();

await builder.Build().RunAsync();
