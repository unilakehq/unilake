using Unilake.Worker.Models.Dbt;

namespace Unilake.Worker.Services.Dbt.Manifest.Event;

public class ManifestCacheProjectAddedEvent
{
    public string ProjectName { get; set; }
    public NodeMetaMap NodeMetaMap { get; set; }
    public MacroMetaMap MacroMetaMap { get; set; }
    public SourceMetaMap SourceMetaMap { get; set; }
    public GraphMetaMap GraphMetaMap { get; set; }
    public TestMetaMap TestMetaMap { get; set; }
    public DocMetaMap DocMetaMap { get; set; }
    public Uri ProjectRoot { get; set; }
}