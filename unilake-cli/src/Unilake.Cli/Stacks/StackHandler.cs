using Pulumi;
using Pulumi.Automation;
using Unilake.Cli.Config;

namespace Unilake.Cli;

public class StackHandler<T> where T : UnilakeStack
{
    private T _unilakeStack;
    private PulumiFn _pulumiFn;
    private WorkspaceStack? _workspaceStack;
    private CancellationTokenRegistration? _registration;

    public StackHandler(T stack)
    {
        _unilakeStack = stack;
        _pulumiFn = PulumiFn.Create(stack.Create);
    }

    public async Task<StackHandler<T>> InitWorkspace(string projectName, string stackName, CancellationToken cancellationToken)
    {
        var stackArgs = new InlineProgramArgs(projectName, stackName, _pulumiFn);
        _workspaceStack = await LocalWorkspace.CreateOrSelectStackAsync(stackArgs, cancellationToken);
        _registration = cancellationToken.Register(() => _workspaceStack.CancelAsync(cancellationToken));
        return this;
    }

    public async Task InstallPluginsAsync(CancellationToken cancellationToken)
    {
        foreach(var p in _unilakeStack.Packages)
            await _workspaceStack.Workspace.InstallPluginAsync(p.name, p.version, PluginKind.Resource, cancellationToken);
    }

    public async Task<UpResult> UpAsync(CancellationToken cancellationToken) => 
    await _workspaceStack?.UpAsync(new UpOptions(), cancellationToken) ?? throw new Exception("Cannot run ");

    public async Task<UpdateResult> DestroyAsync(CancellationToken cancellationToken) => 
    await _workspaceStack?.DestroyAsync(new DestroyOptions(), cancellationToken) ?? throw new Exception("Cannot run ");

    public void Dispose()
    {

    }
}
