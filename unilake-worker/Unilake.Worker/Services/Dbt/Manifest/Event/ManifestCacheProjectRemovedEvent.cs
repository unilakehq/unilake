namespace Unilake.Worker.Services.Dbt.Manifest.Event;

public class ManifestCacheProjectRemovedEvent
{
    public Uri ProjectRoot { get; set; }
}