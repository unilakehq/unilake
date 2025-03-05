using Ardalis.GuardClauses;

namespace Unilake.WebApp.Services;

/// <summary>
/// Main state handler, this class contains and distributes state changes.
/// </summary>
public class StateEventHandler : IDisposable
{
    private readonly Dictionary<string, List<StateChangeRegistration>> _state = new();
    private readonly Dictionary<string, object> _currentState = new();

    public IDisposable RegisterStateHandler(string name, Func<StateChangeEvent, Task> handler)
    {
        Guard.Against.NullOrEmpty(name);
        Guard.Against.Null(handler);

        var registration = new StateChangeRegistration(this, name, handler);
        if (_state.TryGetValue(name, out var handlers))
            handlers.Add(registration);
        else
            _state.Add(name, [registration]);

        return registration;
    }

    public async Task DispatchStateEvent<T>(string name, StateChangeEvent state, bool autoPopulateOldValue = true)
    {
        Guard.Against.NullOrEmpty(name);
        Guard.Against.Null(state);

        if (autoPopulateOldValue && _currentState.ContainsKey(name))
            state = new StateChangeEvent
            {
                NewState = state.NewState,
                OldState = GetState<T>(name)
            };

        if (_state.TryGetValue(name, out var handlers))
            foreach (var registration in handlers)
                await registration.Handler(state);

        _currentState[name] = state.NewState;
    }

    public async Task DispatchStateEvent<T>(string name, Func<T, T> updateStateFunc)
    {
        Guard.Against.NullOrEmpty(name);
        Guard.Against.Null(updateStateFunc);

        var currentState = GetState<T>(name);
        var newState = updateStateFunc(currentState);
        if (newState != null)
            await DispatchStateEvent<T>(name, new StateChangeEvent { NewState = newState, OldState = currentState },
                false);
    }

    public void SetInitialState<T>(string name, T state)
    {
        Guard.Against.NullOrEmpty(name);
        Guard.Against.Null(state);

        if(!_currentState.TryAdd(name, state))
            throw new InvalidOperationException($"State '{name}' already exists");
    }

    public void RemoveStateHandler(string name, Func<StateChangeEvent, Task> handler)
    {
        Guard.Against.NullOrEmpty(name);
        Guard.Against.Null(handler);

        if (!_state.TryGetValue(name, out var handlers)) return;
        var found = handlers.FirstOrDefault(x => x.Handler == handler);
        if (found != null)
            handlers.Remove(found);
    }

    public void Dispose() =>
        _state.Clear();

    public T GetState<T>(string name) => _currentState.TryGetValue(name, out var state)
        ? state is T value ? value : default!
        : default!;
}

public class StateChangeEvent
{
    public object? OldState { get; init; }
    public required object NewState { get; init; }

    public T GetOldState<T>() => OldState != null ? (T)OldState! : default!;
    public T GetNewState<T>() => (T)NewState;
}

sealed class StateChangeRegistration : IDisposable
{
    public readonly Func<StateChangeEvent, Task> Handler;
    private readonly StateEventHandler _stateHandler;
    private readonly string _name;

    public StateChangeRegistration(StateEventHandler stateHandler, string name, Func<StateChangeEvent, Task> handler)
    {
        Guard.Against.Null(stateHandler);
        Guard.Against.NullOrEmpty(name);
        Guard.Against.Null(handler);

        _name = name;
        _stateHandler = stateHandler;
        Handler = handler;
        _stateHandler.RegisterStateHandler(name, Handler);
    }

    public void Dispose() => _stateHandler.RemoveStateHandler(_name, Handler);
}