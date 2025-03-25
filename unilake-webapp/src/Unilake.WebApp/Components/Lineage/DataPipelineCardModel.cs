using Unilake.WebApp.DesignSystem.Components;

namespace Unilake.WebApp.Components;

public class DataPipelineCardModel
{
    public required IIcon SourceIcon { get; init; }
    public required HistoricalStatus.HistoricalStatusItem[] HistoricalStatus { get; init; }
    public string ConnectionName { get; set; }
    public TimeSpan Frequency { get; init; }
    public string ConnectorName { get; set; }
    public DateTime LastUpdated { get; init; }
    public int RecordCount { get; init; }
}