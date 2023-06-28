using System.Threading.Channels;

namespace Unilake.Worker.Services;

public class SequentialTaskProcessor
{
    private readonly Channel<(string, Func<Task>)> _taskQueue;

    public SequentialTaskProcessor() =>
        _taskQueue = Channel.CreateUnbounded<(string, Func<Task>)>();

    public async ValueTask EnqueueTaskAsync((string, Func<Task>) task) => 
        await _taskQueue.Writer.WriteAsync(task).ConfigureAwait(false);

    public async ValueTask<(string, Func<Task>)> DequeueTaskAsync(CancellationToken cancellationToken) =>
        await _taskQueue.Reader.ReadAsync(cancellationToken).ConfigureAwait(false);
}