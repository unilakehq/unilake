using System.Collections.Immutable;
using Pulumi.Automation;
using Spectre.Console;
using Spectre.Console.Rendering;

namespace Unilake.Cli.Stacks;

internal class ResourceState
{
    public string Urn { get; private set; }
    public string? ParentUrn { get; private set; }
    public int Order { get; private set; }
    public OperationType Op { get; private set; }
    public string Type { get; private set; }
    public bool IsDone { get; private set; }
    public IImmutableDictionary<string, object>? Output { get; private set; }
    private string _reportedState = "";

    public ResourceState(string? parentUrn, string urn, int order, OperationType metadataOp, string metadataType)
    {
        ParentUrn = parentUrn;
        Urn = urn;
        Order = order;
        Op = metadataOp;
        Type = metadataType;
    }

    public void SetOutputEventData(IImmutableDictionary<string, object> output)
    {
        IsDone = true;
        Output = output;
    }

    public IRenderable GetStatus(int level = 0)
    {
        var padded_title = new Padder(new Text("Someting long...")).PadRight(16);
        var padded_status = new Padder(new Text($"[green]{CalculatePadding(Urn, level)}[/]"));
        var padded_grid = new Grid();
        padded_grid.AddColumn();
        padded_grid.AddColumn();
        padded_grid.AddRow(padded_title, padded_grid);
        
        var paddedText_I = new Text(Urn);
        var paddedText_II = new Text($"{GetReportedState()}", new Style(Color.Green, decoration: Decoration.Bold));
        
        var pad_I = new Padder(paddedText_I).PadBottom(0).PadTop(0);
        var pad_II = new Padder(paddedText_II).PadLeft(CalculatePadding(Urn, level)).PadBottom(0).PadTop(0);
        var grid = new Grid();
        grid.AddColumn();
        grid.AddColumn();
        grid.AddRow(pad_I, pad_II);
        
        return grid;
    }

    private int CalculatePadding(string title, int level) => Math.Max((120 - (level * 4)) - title.Length, 0);

    private string GetReportedState()
    {
        if (IsDone)
        {
            _reportedState = "Done!";
            return _reportedState;
        }
        
        if (string.IsNullOrWhiteSpace(_reportedState))
        {
            _reportedState = "Creating";
            return _reportedState;
        }

        if (!string.IsNullOrWhiteSpace(_reportedState) && !_reportedState.EndsWith("..."))
            _reportedState += ".";
        else if (_reportedState.EndsWith("..."))
            _reportedState = _reportedState.Substring(0, _reportedState.Length - 3);
        return _reportedState;
    }
}