namespace Unilake.Worker.Services.Dbt.Manifest.Event;

public class ManifestCacheChangedEvent
{
    public List<ManifestCacheProjectAddedEvent> Added { get; set; }
    public List<ManifestCacheProjectRemovedEvent> Removed { get; set; }
}