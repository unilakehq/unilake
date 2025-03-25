using Unilake.WebApp.DesignSystem.Components;

namespace Unilake.WebApp.Components;

public class DataProductCardModel
{
    public required IIcon SourceIcon { get; init; }
    public required HistoricalStatus.HistoricalStatusItem[] HistoricalStatus { get; init; }
    /// <summary>
    /// Source Aligned, or Consumer Aligned
    /// todo: this should be more centralized
    /// </summary>
    public string ArchType { get; init; }
    public TimeSpan Frequency { get; init; }
    /// <summary>
    /// todo: this should be more centralized
    /// </summary>
    public string Availability { get; init; }
    public DateTime LastUpdated { get; init; }
    public int ConsumerCount { get; init; }
}