﻿using Pulumi;
using Pulumi.Automation;
using Pulumi.Automation.Events;
using Unilake.Cli.Config;

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
        var stackArgs = new InlineProgramArgs(projectName, stackName, _pulumiFn);
        _workspaceStack = await LocalWorkspace.CreateOrSelectStackAsync(stackArgs, cancellationToken);
        _registration = cancellationToken.Register(() => _workspaceStack.CancelAsync(cancellationToken));
        return this;
    }

    public async Task InstallPluginsAsync(CancellationToken cancellationToken)
    {
        foreach(var package in _unilakeStack.Packages)
            await _workspaceStack!.Workspace.InstallPluginAsync(package.name, package.version, PluginKind.Resource, cancellationToken);
    }

    public async Task<UpResult> UpAsync(CancellationToken cancellationToken) => 
        await (_workspaceStack?.UpAsync(new UpOptions() { OnEvent = OnEvent }, cancellationToken) ?? throw new Exception("Cannot run "));

    public async Task<UpdateResult> DestroyAsync(CancellationToken cancellationToken) => 
        await (_workspaceStack?.DestroyAsync(new DestroyOptions() { OnEvent = OnEvent }, cancellationToken) ?? throw new Exception("Cannot run "));

    private void OnEvent(EngineEvent @event)
    {
        throw new NotImplementedException();
    }

    public void Dispose()
    {

    }

}
