using System.Collections.Concurrent;
using System.Text;
using Newtonsoft.Json;
using Newtonsoft.Json.Converters;
using Pulumi.Automation;
using Pulumi.Automation.Events;
using Spectre.Console;

namespace Unilake.Cli.Stacks;

internal sealed class StackHandler<T> where T : UnilakeStack
{
    private readonly T _unilakeStack;
    private readonly PulumiFn _pulumiFn;
    private WorkspaceStack? _workspaceStack;
    private CancellationTokenRegistration? _registration;
    private readonly Tree _resourceTree = new("Resources");
    private LiveDisplayContext? _activeCtx;
    private readonly ConcurrentDictionary<string, ResourceState> _resourceStates = new();

    public StackHandler(T stack)
    {
        _unilakeStack = stack;
        _pulumiFn = PulumiFn.Create(stack.Create);
    }

    public async Task<StackHandler<T>> InitWorkspace(string projectName, string stackName, CancellationToken cancellationToken)
    {
        try
        {
            var stackArgs = new InlineProgramArgs(projectName, stackName, _pulumiFn)
            {
                ProjectSettings = new ProjectSettings(projectName, ProjectRuntimeName.Dotnet)
                {
                    Name = stackName,
                    Backend = new ProjectBackend
                    {
                        Url = "file://~"
                    }
                },
                EnvironmentVariables = new Dictionary<string, string?>
                {
                    {"PULUMI_CONFIG_PASSPHRASE", "some-passphrase"}
                }
            };
            _workspaceStack = await LocalWorkspace.CreateOrSelectStackAsync(stackArgs, cancellationToken);
            _registration = cancellationToken.Register(() => _workspaceStack.CancelAsync(cancellationToken));
        }
        catch (Exception e)
        {
            Console.WriteLine(e);
            throw new CliException("Cancelled");
        }
        return this;
    }

    public async Task InstallPluginsAsync(CancellationToken cancellationToken)
    {
        foreach (var package in _unilakeStack.Packages)
            await _workspaceStack!.Workspace.InstallPluginAsync(package.name, package.version, PluginKind.Resource, cancellationToken);
    }

    public async Task<UpResult?> UpAsync(CancellationToken cancellationToken)
    {
        if (_workspaceStack == null)
            throw new CliException("WorkspaceStack uninitialized");

        UpResult? result = null;
        AnsiConsole.WriteLine("");
        await AnsiConsole.Live(_resourceTree).StartAsync(async ctx =>
        {
            _activeCtx = ctx;
            var upTask = _workspaceStack.UpAsync(new UpOptions { OnEvent = OnEvent, Parallel = 4 }, cancellationToken);
            var reportingTask = Task.Run(async () =>
            {
                while (!upTask.IsCompleted)
                {
                    UpdateTree();
                    await Task.Delay(500, cancellationToken);
                }
            });

            result = await upTask;
            await reportingTask;
            UpdateTree();
        });
        return result;
    }

    public async Task<UpdateResult?> DestroyAsync(CancellationToken cancellationToken)
    {
        if (_workspaceStack == null)
            throw new CliException("WorkspaceStack uninitialized");

        UpdateResult? result = null;
        AnsiConsole.WriteLine("");
        await AnsiConsole.Live(_resourceTree).StartAsync(async ctx =>
        {
            _activeCtx = ctx;
            var destroyStack = _workspaceStack.DestroyAsync(new DestroyOptions { OnEvent = OnEvent, Parallel = 4 }, cancellationToken);
            var reportingTask = Task.Run(async () =>
            {
                while (!destroyStack.IsCompleted)
                {
                    UpdateTree();
                    await Task.Delay(500, cancellationToken);
                }
            });

            result = await destroyStack;
            await reportingTask;
            UpdateTree();
        });
        return result;
    }

    public async Task CancelAsync(CancellationToken cancellationToken) =>
        await (_workspaceStack?.CancelAsync(cancellationToken) ?? throw new CliException("Cannot run "));

    private void UpdateTree()
    {
        if (_activeCtx == null)
            throw new CliException("No active context, expected active context to exist");

        var nodes = _resourceStates.Where(x => string.IsNullOrWhiteSpace(x.Value.ParentUrn))
            .Where(x => x.Value.HasChildResourcesWithChanges())
            .OrderBy(x => x.Value.Order).ToArray();

        if (_resourceTree.Nodes.Count > 0)
            _resourceTree.Nodes.Clear();

        if (!nodes.Any())
        {
            _resourceTree.AddNode(new Text("No changes found...", new Style(Color.Orange1)));
            return;
        }

        foreach (var (k, state) in nodes)
            AddTreeNodes(k, state);

        _activeCtx.Refresh();
    }

    private void AddTreeNodes(string root, ResourceState state, int level = 0, TreeNode? parent = null)
    {
        if ((state.HasChildResources && !state.HasChildResourcesWithChanges()) || state is { ReportableOperation: false, HasChildResources: false })
            return;

        var consoleText = state.GetStatus(level);
        TreeNode current = parent == null
            ? _resourceTree.AddNode(consoleText)
            : parent.AddNode(consoleText);

        level += 1;
        foreach (var (k, childState) in _resourceStates.Where(x => x.Value.ParentUrn == root).OrderBy(x => x.Value.Order))
            AddTreeNodes(k, childState, level, current);
    }

    private void OnEvent(EngineEvent onEvent)
    {
        var export = Environment.GetEnvironmentVariable("DEBUG_EXPORT_PULUMI_EVENTS");
        if (!string.IsNullOrWhiteSpace(export) && string.Equals(export, "true", StringComparison.InvariantCultureIgnoreCase))
            ExportEventDetails(onEvent);

        string urn;
        ResourceState resourceState;
        switch (onEvent.AsType())
        {
            case EngineEventType.DiagnosticEvent:
                break;
            case EngineEventType.CancelEvent:
                break;
            case EngineEventType.PolicyEvent:
                break;
            case EngineEventType.PreludeEvent:
                break;
            case EngineEventType.SummaryEvent:
                break;
            case EngineEventType.ResourceOutputsEvent:
                if (onEvent.ResourceOutputsEvent == null)
                    break;
                urn = onEvent.ResourceOutputsEvent.Metadata.Urn;
                resourceState = _resourceStates[urn];
                resourceState.SetOutputEventData(onEvent.ResourceOutputsEvent.Metadata.Old?.Outputs, onEvent.ResourceOutputsEvent.Metadata.New?.Outputs);
                break;
            case EngineEventType.ResourcePreEvent:
                if (onEvent.ResourcePreEvent == null)
                    break;
                urn = onEvent.ResourcePreEvent.Metadata.Urn;
                var parentUrn = onEvent.ResourcePreEvent.Metadata.New?.Parent ??
                            onEvent.ResourcePreEvent.Metadata.Old?.Parent ?? string.Empty;
                resourceState = new ResourceState(
                    parentUrn,
                    urn,
                    onEvent.Sequence,
                    onEvent.ResourcePreEvent.Metadata.Op,
                    onEvent.ResourcePreEvent.Metadata.Type);
                _resourceStates[urn] = resourceState;
                if (_resourceStates.TryGetValue(parentUrn, out var parentResource))
                    parentResource.AddChildResourceState(resourceState);
                break;
            case EngineEventType.StandardOutputEvent:
                break;
            case EngineEventType.ResourceOperationFailedEvent:
                break;
            default:
                throw new ArgumentOutOfRangeException(nameof(onEvent));
        }
    }

    public void ReportUpSummary(UpResult upResult)
    {
        // TODO: show updates on resources and resource information
        AnsiConsole.WriteLine();
        switch (upResult.Summary.Result)
        {
            case UpdateState.Succeeded:
                AnsiConsole.MarkupLine(string.Format(Message.ReportUpSummaryResultStatus, ":check_mark_button:", "Success :rocket::rocket::rocket:"));
                break;
            case UpdateState.Failed:
                AnsiConsole.MarkupLine(string.Format(Message.ReportUpSummaryResultStatus, ":cross_mark:", "Failed"));
                break;
            default:
                throw new ArgumentOutOfRangeException(nameof(upResult));
        }
    }

    public void ReportDestroySummary(UpdateResult updateResult)
    {
        AnsiConsole.WriteLine();
        switch (updateResult.Summary.Result)
        {
            case UpdateState.Succeeded:
                AnsiConsole.MarkupLine(string.Format(Message.ReportUpSummaryResultStatus, ":check_mark_button:", "Success"));
                break;
            case UpdateState.Failed:
                AnsiConsole.MarkupLine(string.Format(Message.ReportUpSummaryResultStatus, ":cross_mark:", "Failed"));
                break;
            default:
                throw new ArgumentOutOfRangeException(nameof(updateResult));
        }
    }

    private void ExportEventDetails(EngineEvent onEvent)
    {
        StringBuilder sb = new StringBuilder();
        JsonSerializerSettings settings = new JsonSerializerSettings
        {
            Converters = new List<JsonConverter> { new StringEnumConverter() }
        };
        string path = Enum.GetName(typeof(EngineEventType), onEvent.AsType()) ?? "unknown";
        switch (onEvent.AsType())
        {
            case EngineEventType.DiagnosticEvent:
                //sb.Append(JsonConvert.SerializeObject(onEvent.DiagnosticEvent, Formatting.Indented, settings));
                return;
            case EngineEventType.CancelEvent:
                sb.Append(JsonConvert.SerializeObject(onEvent.CancelEvent, Formatting.Indented, settings));
                break;
            case EngineEventType.PolicyEvent:
                sb.Append(JsonConvert.SerializeObject(onEvent.PolicyEvent, Formatting.Indented, settings));
                break;
            case EngineEventType.PreludeEvent:
                sb.Append(JsonConvert.SerializeObject(onEvent.PreludeEvent, Formatting.Indented, settings));
                break;
            case EngineEventType.SummaryEvent:
                sb.Append(JsonConvert.SerializeObject(onEvent.SummaryEvent, Formatting.Indented, settings));
                break;
            case EngineEventType.ResourceOutputsEvent:
                sb.Append(JsonConvert.SerializeObject(onEvent.ResourceOutputsEvent, Formatting.Indented, settings));
                break;
            case EngineEventType.ResourcePreEvent:
                sb.Append(JsonConvert.SerializeObject(onEvent.ResourcePreEvent, Formatting.Indented, settings));
                break;
            case EngineEventType.StandardOutputEvent:
                sb.Append(JsonConvert.SerializeObject(onEvent.StandardOutputEvent, Formatting.Indented, settings));
                break;
            case EngineEventType.ResourceOperationFailedEvent:
                sb.Append(JsonConvert.SerializeObject(onEvent.ResourceOperationFailedEvent, Formatting.Indented, settings));
                break;
            default:
                throw new ArgumentOutOfRangeException(nameof(onEvent));
        }
        path = Path.Combine(Directory.GetCurrentDirectory(), $"{onEvent.Sequence}-{path}-output.json");
        File.WriteAllText(path, sb.ToString());
    }
}