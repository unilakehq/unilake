using System.Collections.Immutable;
using Pulumi.Automation;
using Spectre.Console;
using Spectre.Console.Rendering;

namespace Unilake.Cli.Stacks;

internal class ResourceState
{
    private readonly OperationType[] Reportableoperations = new[] { OperationType.Create, OperationType.Delete, OperationType.Replace, OperationType.Update };
    public string Urn { get; private set; }
    public string? ParentUrn { get; private set; }
    public int Order { get; private set; }
    public OperationType Op { get; private set; }
    public string Type { get; private set; }
    public bool IsDone { get; private set; }
    public bool HasChildResources => _childResources.Count > 0;
    public bool ReportableOperation => Reportableoperations.Contains(Op);
    public IImmutableDictionary<string, object>? OutputNew { get; private set; }
    public IImmutableDictionary<string, object>? OutputOld { get; private set; }
    private string _reportedState = "";
    private readonly List<ResourceState> _childResources = new();

    public ResourceState(string? parentUrn, string urn, int order, OperationType metadataOp, string metadataType)
    {
        ParentUrn = parentUrn;
        Urn = urn;
        Order = order;
        Op = metadataOp;
        Type = metadataType;
    }

    public void AddChildResourceState(ResourceState child) => _childResources.Add(child);

    public void SetOutputEventData(IImmutableDictionary<string, object>? old, IImmutableDictionary<string, object>? @new)
    {
        IsDone = true;
        OutputNew = @new;
        OutputOld = old;
    }

    public IRenderable GetStatus(int level = 0)
    {
        string title = GetResourceName();
        var colored = Color.Green;
        var titleText = new Padder(new Text(title)).PadBottom(0).PadTop(0);
        var statusText = new Padder(new Text($"{GetReportedState()}", new Style(colored, decoration: Decoration.Bold))).PadLeft(CalculatePadding(title, level)).PadBottom(0).PadTop(0);
        var grid = new Grid();
        grid.AddColumn();
        grid.AddColumn();
        grid.AddRow(titleText, statusText);

        return grid;
    }

    public bool HasChildResourcesWithChanges()
    {
        bool ChildWithChanges(ResourceState[] states)
        {
            foreach (var state in states)
            {
                if (state.ReportableOperation)
                    return true;
                if (state._childResources.Count > 0 && ChildWithChanges(state._childResources.ToArray()))
                    return true;
            }

            return false;
        }

        return ChildWithChanges(_childResources.ToArray());
    }

    private string GetResourceName() => Urn.Split(new[] { "::" }, StringSplitOptions.None)[^2]
            .Replace("pulumi:", string.Empty)
            .Replace("Stack", "UniLake");

    private int CalculatePadding(string title, int level) => Math.Max((90 - (level * 4)) - title.Length, 0);

    private string GetReportedState()
    {
        if (IsDone)
        {
            _reportedState = !HasChildResources ? "Done!" : "";
            return _reportedState;
        }
        else if (HasChildResources)
            return "";

        if (string.IsNullOrWhiteSpace(_reportedState))
        {
            _reportedState = Op switch
            {
                OperationType.Read => "Reading",
                OperationType.Create => "Creating",
                OperationType.Delete => "Deleting",
                OperationType.Update => "Updating",
                OperationType.Same => "Verifying",
                _ => Enum.GetName(typeof(OperationType), Op) ?? "Unknown"
            };
            return _reportedState;
        }

        if (!string.IsNullOrWhiteSpace(_reportedState) && !_reportedState.EndsWith("..."))
            _reportedState += ".";
        else if (_reportedState.EndsWith("..."))
            _reportedState = _reportedState.Substring(0, _reportedState.Length - 3);
        return _reportedState;
    }

    public void UpdateOperation(OperationType op) => Op = op;
}