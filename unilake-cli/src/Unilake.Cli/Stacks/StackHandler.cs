using Pulumi.Automation;
using Pulumi.Automation.Events;

namespace Unilake.Cli;

public sealed class StackHandler<T> where T : UnilakeStack
{
    private readonly T _unilakeStack;
    private readonly PulumiFn _pulumiFn;
    private WorkspaceStack? _workspaceStack;
    private CancellationTokenRegistration? _registration;

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
            throw new Exception("Cancelled");
        }
        return this;
    }

    public async Task InstallPluginsAsync(CancellationToken cancellationToken)
    {
        foreach(var package in _unilakeStack.Packages)
            await _workspaceStack!.Workspace.InstallPluginAsync(package.name, package.version, PluginKind.Resource, cancellationToken);
    }

    public async Task<UpResult> UpAsync(CancellationToken cancellationToken) => 
        await (_workspaceStack?.UpAsync(new UpOptions { OnEvent = OnEvent }, cancellationToken) ?? throw new Exception("Cannot run "));

    public async Task<UpdateResult> DestroyAsync(CancellationToken cancellationToken) => 
        await (_workspaceStack?.DestroyAsync(new DestroyOptions { OnEvent = OnEvent }, cancellationToken) ?? throw new Exception("Cannot run "));

    private void OnEvent(EngineEvent @event)
    {
        Console.WriteLine(@event.ToString());
    }

    public void Dispose()
    {

    }

}
