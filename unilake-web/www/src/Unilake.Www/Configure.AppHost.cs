[assembly: HostingStartup(typeof(Unilake.Www.AppHost))]

namespace Unilake.Www;

public class AppHost : AppHostBase, IHostingStartup
{
    public void Configure(IWebHostBuilder builder) => builder
        .ConfigureServices(services => {
            // Configure ASP.NET Core IOC Dependencies
        });
    public override void Configure(Funq.Container container)
    {
    }
    public AppHost() : base("Unilake.Www", typeof(MyServices).Assembly) {}
}

public class Hello : IReturn<StringResponse> {}
public class MyServices : Service
{
    public object Any(Hello request) => new StringResponse { Result = $"Hello, World!" };
}
