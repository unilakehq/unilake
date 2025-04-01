namespace Unilake.WebApp.DesignSystem.Services.Messaging;

public abstract class MessageService
{
    /// <summary>
    /// Message channels and their callbacks.
    /// </summary>
    protected readonly MessageHandler MessageHandler = new();

    /// <summary>
    /// Dispatches a message to the specified channel.
    /// </summary>
    /// <param name="channelName">Name of the channel to dispatch to</param>
    /// <param name="message">Message to dispatch</param>
    protected async Task DispatchMessageAsync(string channelName, ServerMessage message) => await MessageHandler.DispatchMessage(channelName, message);

    /// <summary>
    /// Registers a callback for a specific message channel.
    /// </summary>
    /// <param name="channelName">Name of the channel to subscribe to</param>
    /// <param name="callback">Function to execute on message retrieval</param>
    /// <returns></returns>
    public IDisposable RegisterMessageHandler(string channelName, Func<ServerMessage, Task> callback) => MessageHandler.RegisterMessageHandler(channelName, callback);

    /// <summary>
    /// Registers a callback for a firehose (all channels) of messages.
    /// </summary>
    /// <param name="callback">Function to execute on message retrieval</param>
    /// <returns>An IDisposable, which when disposed cleans up this callback function</returns>
    public IDisposable RegisterFirehoseHandler(Func<ServerMessage, Task> callback) =>
        RegisterMessageHandler(MessageHandler.FirehoseChannel, callback);

    /// <summary>
    /// Establishes a connection to the message service.
    /// </summary>
    public abstract Task ConnectAsync();
}