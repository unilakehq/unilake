using Volo.Abp.DependencyInjection;
using Volo.Abp.Ui.Branding;

namespace Unilake.WebApi;

[Dependency(ReplaceServices = true)]
public class WebApiBrandingProvider : DefaultBrandingProvider
{
    public override string AppName => "WebApi";
}
