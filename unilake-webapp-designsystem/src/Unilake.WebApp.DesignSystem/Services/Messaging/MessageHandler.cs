using Ardalis.GuardClauses;

namespace Unilake.WebApp.DesignSystem.Services.Messaging;

/// <summary>
/// Main message handler, this class contains and distributes messages.
/// </summary>
public class MessageHandler : IDisposable
{
    public static string FirehoseChannel = "*";
    private readonly Dictionary<string, List<MessageHandlerRegistration>> _handlers = new();
    
    public IDisposable RegisterMessageHandler(string channelName, Func<ServerMessage, Task> handler)
    {
        Guard.Against.NullOrEmpty(channelName);
        Guard.Against.Null(handler);

        var registration = new MessageHandlerRegistration(this, channelName, handler);
        if (_handlers.TryGetValue(channelName, out var handlers))
            handlers.Add(registration);
        else
            _handlers.Add(channelName, [registration]);
        return registration;
    }
    
    public void RemoveMessageHandler(string channelName, Func<ServerMessage, Task> handler)
    {
        Guard.Against.NullOrEmpty(channelName);
        Guard.Against.Null(handler);
        
        if(!_handlers.TryGetValue(channelName, out var handlers)) return;
        var found = handlers.FirstOrDefault(x => x.Handler == handler);
        if (found!= null)
            handlers.Remove(found);
    }

    public async Task DispatchMessage(string channelName, ServerMessage message)
    {
        Guard.Against.NullOrEmpty(channelName);
        Guard.Against.Null(message);

        if (_handlers.TryGetValue(channelName, out var channelHandlers) |
            _handlers.TryGetValue(FirehoseChannel, out var firehoseHandlers))
        {
            channelHandlers ??= [];
            firehoseHandlers ??= [];

            foreach (var registration in channelHandlers.Concat(firehoseHandlers))
                await registration.Handler(message);
        }
    }

    public void Dispose() => _handlers.Clear();
}

sealed class MessageHandlerRegistration : IDisposable
{
    public readonly Func<ServerMessage, Task> Handler;
    private readonly MessageHandler _messageHandler;
    private readonly string _channelName;

    public MessageHandlerRegistration(MessageHandler messageHandler, string channelName, Func<ServerMessage, Task> handler)
    {
        Guard.Against.Null(messageHandler);
        Guard.Against.NullOrEmpty(channelName);
        Guard.Against.Null(handler);

        _channelName = channelName;
        _messageHandler = messageHandler;
        Handler = handler;
    }

    public void Dispose() =>
        _messageHandler.RemoveMessageHandler(_channelName, Handler);
}

public class ServerMessage(string ChannelName, string Message)
{
    private object? _message;
    // public T GetMessage<T>() => _message != null? (T)_message :
    //    _message = JsonSerializer.Deserialize<T>(Message) ?? default!;
}
