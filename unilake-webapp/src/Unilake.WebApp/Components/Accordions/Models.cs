using System.ComponentModel;
namespace Unilake.WebApp.Components;

public class IntegrationPipelineEntitySelectionModel
{
    public IntegrationStatus.IntegrationStatusIndicator Status { get; set; } =
        IntegrationStatus.IntegrationStatusIndicator.Added;
    public required string EntityName { get; init; }
    public required Dictionary<string, IntegrationPipelineEntityAttribute> Attributes { get; init; }
    public bool IsIncluded { get; set; }
    public (int, int) SelectedCount => (Attributes.Values.Count(x => x.IsIncluded ?? false), Attributes.Values.Count);
    public IntegrationPipelineRunType RunType { get; set; } = IntegrationPipelineRunType.FullOverwrite;
    public bool RequiresCursor => RunType switch
    {
        IntegrationPipelineRunType.IncrementalDedup => true,
        IntegrationPipelineRunType.IncrementalAppend => true,
        _ => false
    };
    public IntegrationPipelineEntityAttribute? CursorField => Attributes.Values.FirstOrDefault(x => x.IsCursor);
    public IEnumerable<KeyValuePair<string, IntegrationPipelineEntityAttribute>> PrimaryKeys => Attributes.Where(x => x.Value.IsPrimaryKey ?? false);
}

public class IntegrationPipelineEntityAttribute
{
    public bool? IsIncluded { get; set; } = true;
    public required string DataType { get; init; }
    public bool IsCursor { get; set; } = false;
    public bool? IsPrimaryKey { get; set; } = false;
    public bool? IsHashColumn { get; set; } = false;
    public IntegrationPipelineEntityAttribute[]? Children { get; init; }
}

public enum IntegrationPipelineRunType
{
    [Description("Full Refresh | Overwrite")]
    FullOverwrite,
    [Description("Full Refresh | Append")]
    FullAppend,
    [Description("Full Refresh | Overwrite + Dedup")]
    FullOverwriteDedup,
    [Description("Incremental | Append + Dedup")]
    IncrementalDedup,
    [Description("Incremental | Append")]
    IncrementalAppend,
}

enum DataType
{
    String,
    Integer,
    Object,
    TimeStamp
}