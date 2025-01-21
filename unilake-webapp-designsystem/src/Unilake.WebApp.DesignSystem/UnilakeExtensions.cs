using Microsoft.Extensions.DependencyInjection;
using Unilake.WebApp.DesignSystem.Services;

namespace Unilake.WebApp.DesignSystem;

public static class UnilakeExtensions
{
    public static IServiceCollection AddUnilakeDesignSystem(this IServiceCollection services) =>
        services.AddScoped<UnilakeService>()
            .AddScoped<ToastService>()
            .AddScoped<IModalService, ModalService>();
}