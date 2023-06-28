using System.Threading.Channels;
using OneOf;
using OneOf.Types;
using Unilake.Worker.Contracts;
using Unilake.Worker.Contracts.Responses;
using Unilake.Worker.Services;

namespace Unilake.Worker.Events;

public abstract class TaskEventHandler<T> where T : IServiceTaskEvent
{
    private readonly IProcessManager _manager;
    private readonly ILogger _logger;
    private readonly SequentialTaskProcessor _taskProcessor;
    private readonly ChannelWriter<EventStreamResponse> _writer;
    protected string CurrentClassName { get; init; }

    protected TaskEventHandler(IProcessManager manager, SequentialTaskProcessor sequentialTaskProcessor, ChannelWriter<EventStreamResponse> writer,
        ILogger<TaskEventHandler<T>> logger)
    {
        _manager = manager;
        _taskProcessor = sequentialTaskProcessor;
        _logger = logger;
        _writer = writer;
    }

    protected void LogEventModel(string message, T eventModel) =>
        _logger.LogInformation("{Message} - {EventModelProcessReferenceId}", message, eventModel.ProcessReferenceId);

    protected void SetResultStatusInProgress(T eventModel) =>
        _manager.SetResultStatus(eventModel.ProcessReferenceId, ResultStatus.InProgress,
            eventModel.OnInProgressMessage);

    protected async Task HandleResult(OneOf<Success<IRequestResponse>, Error<string>> result, T eventModel)
    {
        result.Switch(
            success =>
            {
                LogEventModel($"Executing {CurrentClassName} - Success!", eventModel);
                _manager.SetSuccessResponse(eventModel.ProcessReferenceId, success);
            },
            error =>
            {
                _logger.LogError("Executing {CurrentClassName} - {ProcessReferenceId} - Failed: {Error}",
                    CurrentClassName,
                    eventModel.ProcessReferenceId,
                    error.Value);
                _manager.SetErrorResponse(eventModel.ProcessReferenceId, error);
            });

        await WriteResultAsync(eventModel);
    }

    private OneOf<Success<IRequestResponse>, Error<Exception>> GetRequestResponse(T eventModel) =>
        _manager.Status<IRequestResponse>(eventModel.ProcessReferenceId);

    protected async Task EnqueueTaskAsync(string processReferenceId, Func<Task> taskFunc) =>
        await _taskProcessor.EnqueueTaskAsync((processReferenceId, taskFunc)).ConfigureAwait(false);

    private async Task WriteResultAsync(T eventModel)
    {
        var found = GetRequestResponse(eventModel);
        if (found.IsT0)
        {
            await _writer.WriteAsync(new EventStreamRequestResponse(found.AsT0.Value));
            return;
        }
        
        _logger.LogError(found.AsT1.Value, "Failed to write result");
    }
}