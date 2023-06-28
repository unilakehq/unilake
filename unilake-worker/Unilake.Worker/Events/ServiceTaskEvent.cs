using OneOf;
using OneOf.Types;
using Unilake.Worker.Contracts;

namespace Unilake.Worker.Events;

public interface IServiceTaskEvent
{
    string ProcessReferenceId { get; set; }
    bool RunAsync { get; set; }
    string OnCancelledMessage { get; set; }
    string OnInProgressMessage { get; set; }
}

public abstract class ServiceTaskEvent<T> : IEvent, IServiceTaskEvent
{
    public string ProcessReferenceId { get; set; }
    public bool RunAsync { get; set; } = true;
    public string OnCancelledMessage { get; set; } = "Cancelled";
    public string OnInProgressMessage { get; set; } = "In progress";

    public ServiceTaskEvent<T> SetProcessReferenceId(string processReferenceId)
    {
        ProcessReferenceId = processReferenceId;
        return this;
    }
    
    public ServiceTaskEvent<T> SetRunAsync(bool runAsync)
    {
        RunAsync = runAsync;
        return this;
    }

    public ServiceTaskEvent<T> SetOnCancelledMessage(string message)
    {
        OnCancelledMessage = message;
        return this;
    }

    public ServiceTaskEvent<T> SetOnInProgressMessage(string message)
    {
        OnInProgressMessage = message;
        return this;
    }

    protected virtual OneOf<Success<IRequestResponse>, Error<string>> Handle(T service) => 
        new Error<string>("No handle method defined");

    public virtual Task<OneOf<Success<IRequestResponse>, Error<string>>> HandleAsync(T service) =>
        Task.FromResult(Handle(service));
}