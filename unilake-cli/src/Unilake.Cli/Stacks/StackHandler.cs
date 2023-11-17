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
    private readonly Tree _resourceTree = new ("Resources");
    private LiveDisplayContext? _activeCtx;
    private readonly Dictionary<string, ResourceState> _resourceStates = new ();

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
        foreach(var package in _unilakeStack.Packages)
            await _workspaceStack!.Workspace.InstallPluginAsync(package.name, package.version, PluginKind.Resource, cancellationToken);
    }

    public async Task<UpResult?> UpAsync(CancellationToken cancellationToken)
    {
        if (_workspaceStack == null)
            throw new CliException("WorkspaceStack uninitialized");

        UpResult? result = null;
        AnsiConsole.WriteLine();
        AnsiConsole.WriteLine();
        await AnsiConsole.Live(_resourceTree).StartAsync(async ctx =>
        {
            _activeCtx = ctx;
            var upTask = _workspaceStack.UpAsync(new UpOptions { OnEvent = OnEvent }, cancellationToken);
            var reportingTask = Task.Run(async () =>
            {
                while (!upTask.IsCompleted)
                {
                    UpdateTree();
                    await Task.Delay(500);
                }
            });

            await upTask;
            await reportingTask;
            UpdateTree();
        });
        return result;
    }

    public async Task<UpdateResult> DestroyAsync(CancellationToken cancellationToken) => 
        await (_workspaceStack?.DestroyAsync(new DestroyOptions { OnEvent = OnEvent }, cancellationToken) ?? throw new CliException("Cannot run "));

    public async Task CancelAsync(CancellationToken cancellationToken) => 
        await (_workspaceStack?.CancelAsync(cancellationToken) ?? throw new CliException("Cannot run "));
    
    private void UpdateTree()
    {
        if (_activeCtx == null)
            throw new CliException("No active context, expected active context to exist");
        _resourceTree.Nodes.Clear();
        foreach (var (k, state) in _resourceStates.Where(x => string.IsNullOrWhiteSpace(x.Value.ParentUrn)).OrderBy(x => x.Value.Order))
            AddTreeNodes(k, state);
        _activeCtx.Refresh();
    }

    private void AddTreeNodes(string root, ResourceState state, int level = 0, TreeNode? parent = null)
    {
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
        if(!string.IsNullOrWhiteSpace(export) && string.Equals(export, "true", StringComparison.InvariantCultureIgnoreCase))
            ExportEventDetails(onEvent);

        string urn;
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
                if(onEvent.ResourceOutputsEvent == null)
                    break;
                urn = onEvent.ResourceOutputsEvent.Metadata.Urn;
                var resourceState = _resourceStates[urn];
                if(onEvent.ResourceOutputsEvent.Metadata.New != null)
                    resourceState.SetOutputEventData(onEvent.ResourceOutputsEvent.Metadata.New.Outputs);
                break;
            case EngineEventType.ResourcePreEvent:
                if (onEvent.ResourcePreEvent == null)
                    break;
                urn = onEvent.ResourcePreEvent.Metadata.Urn;
                _resourceStates.Add(urn, new ResourceState(onEvent.ResourcePreEvent.Metadata.New?.Parent ?? string.Empty, 
                    urn, onEvent.Sequence, onEvent.ResourcePreEvent.Metadata.Op, onEvent.ResourcePreEvent.Metadata.Type));
                break;
            case EngineEventType.StandardOutputEvent:
                break;
            case EngineEventType.ResourceOperationFailedEvent:
                break;
            default:
                throw new ArgumentOutOfRangeException(nameof(onEvent));
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
                sb.Append(JsonConvert.SerializeObject(onEvent.DiagnosticEvent, Formatting.Indented, settings));
                break;
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
