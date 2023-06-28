using OneOf.Types;
using Prometheus;
using Unilake.Worker.Contracts;
using Unilake.Worker.Contracts.Responses;

namespace Unilake.Worker.Services;

public class SequentialTaskProcessorHostedService<T> : BackgroundService
{
    private readonly SequentialTaskProcessor _taskProcessor;
    private readonly IProcessManager _processManager;
    private readonly ILogger<SequentialTaskProcessorHostedService<T>> _logger;

    public SequentialTaskProcessorHostedService(
        SequentialTaskProcessor taskProcessor,
        IProcessManager processManager,
        ILogger<SequentialTaskProcessorHostedService<T>> logger)
    {
        _taskProcessor = taskProcessor;
        _processManager = processManager;
        _logger = logger;
    }

    protected override async Task ExecuteAsync(CancellationToken ct)
    {
        while (!ct.IsCancellationRequested)
        {
            var (processReferenceId, task) = await _taskProcessor.DequeueTaskAsync(ct).ConfigureAwait(false);
            try
            {
                var statusResult = _processManager.Status<IRequestResponse>(processReferenceId);
                var process = statusResult.Match(s => s.Value, _ => null);
                switch (process)
                {
                    case { Status: ResultStatus.Queued }:
                        await task().ConfigureAwait(false);
                        break;
                    // TODO: set the cancel status
                    // case { Status: ResultStatus.Cancelled }:
                    //     process.Message = taskModel.OnCancelledMessage;
                    //     break;
                }
            }
            catch (Exception ex)
            {
                _processManager.SetErrorResponse(processReferenceId, new Error<string>("Internal Server Error"));
                _logger.LogError(ex,
                    "Error occurred while processing task with ProcessReferenceId: {ProcessReferenceId}",
                    processReferenceId);
            }
            finally
            {
                var executionCount = Metrics.CreateCounter(
                    string.Format(WorkerMetrics.HostedserviceSequentialtaskprocessorExecutionsTotal, typeof(T).Name.ToLower()),
                    WorkerMetrics.HostedserviceSequentialtaskprocessorExecutionsTotalDesc);
                executionCount.Inc();
            }
        }
    }
    
}