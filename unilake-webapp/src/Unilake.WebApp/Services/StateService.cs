namespace Unilake.WebApp.Services;

public abstract class StateService
{
    /// <summary>
    /// State as stored in the application (singleton)
    /// </summary>
    protected readonly StateEventHandler StateHandler = new();

    /// <summary>
    /// Dispatch a state change event to all registered handlers for the specified state.
    /// </summary>
    /// <param name="name">Name of the state to dispatch</param>
    /// <param name="state">State changes, both old and new state</param>
    /// <param name="autoPopulateOldValue">Automatically populate the old state with the current state if true</param>
    public Task DispatchStateEvent<T>(string name, StateChangeEvent state, bool autoPopulateOldValue = true) => StateHandler.DispatchStateEvent<T>(name, state, autoPopulateOldValue);

    /// <summary>
    /// Dispatch a state change event to all registered handlers for the specified state, updating the state using the provided function.
    /// </summary>
    /// <param name="name">Name of the state to dispatch</param>
    /// <param name="updateStateFunc">Function to change the state in place</param>
    /// <typeparam name="T">Type of the state stored</typeparam>
    public Task DispatchStateEvent<T>(string name, Func<T, T> updateStateFunc) => StateHandler.DispatchStateEvent(name, updateStateFunc);

    /// <summary>
    /// Register a state change handler for a specific state. Returns an IDisposable to unregister the handler when no longer needed.
    /// </summary>
    /// <param name="name">Name of the state to track</param>
    /// <param name="handler">Function to invoke on state changes</param>
    public IDisposable RegisterStateHandler(string name, Func<StateChangeEvent, Task> handler) => StateHandler.RegisterStateHandler(name, handler);

    /// <summary>
    /// Get the current value for a specific state.
    /// </summary>
    /// <param name="name">Name of the state</param>
    /// <typeparam name="T">Expected state type</typeparam>
    /// <returns></returns>
    public T GetState<T>(string name) => StateHandler.GetState<T>(name);
}