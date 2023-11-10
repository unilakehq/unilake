[assembly: HostingStartup(typeof(Unilake.Docs.AppHost))]

namespace Unilake.Docs;

public class AppHost : AppHostBase, IHostingStartup
{
    public void Configure(IWebHostBuilder builder) => builder
        .ConfigureServices(services => {
            // Configure ASP.NET Core IOC Dependencies
        });

    public override void Configure(Funq.Container container)
    {
        ConfigurePlugin<NativeTypesFeature>(feature =>
        {
            feature.MetadataTypesConfig.ExportAttributes.Add(typeof(FieldAttribute));
        });
    }
    
    public AppHost() : base("Unilake.Docs", typeof(MyServices).Assembly) {}
}

public class Hello : IReturn<StringResponse> {}
public class MyServices : Service
{
    public object Any(Hello request) => new StringResponse { Result = $"Hello, World!" };
}
