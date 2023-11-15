using System.Collections.Immutable;
using Pulumi.Automation;
using Spectre.Console;
using Spectre.Console.Rendering;


namespace Unilake.Cli.Stacks;

public class ResourceState
{
    public string Urn { get; private set; }
    public string? ParentUrn { get; private set; }
    public int Order { get; private set; }
    public OperationType Op { get; private set; }
    public string Type { get; private set; }
    public bool IsDone { get; private set; }
    public IImmutableDictionary<string, object>? Output { get; private set; }

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
        var padded_status = new Padder(new Text("[green]Ok[/]"));
        var padded_grid = new Grid();
        padded_grid.AddColumn();
        padded_grid.AddColumn();
        padded_grid.AddRow(padded_title, padded_grid);
        
        var paddedText_I = new Text(Urn);
        var paddedText_II = new Text("Ok", new Style(Color.Green, decoration: Decoration.Bold));
        
        var pad_I = new Padder(paddedText_I).PadRight(CalculatePadding(Urn, level)).PadBottom(0).PadTop(0);
        var pad_II = new Padder(paddedText_II).PadBottom(0).PadTop(0);
        var grid = new Grid();
        grid.AddColumn();
        grid.AddColumn();
        grid.AddRow(pad_I, pad_II);
        
        return grid;
    }

    private int CalculatePadding(string title, int level) => Math.Max(180 - title.Length - (level * 2), 0);
}