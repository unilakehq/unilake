using System.Threading.Channels;
using Unilake.Worker.Contracts.Responses;
using Unilake.Worker.Services;
using Unilake.Worker.Services.Dbt;

namespace Unilake.Worker.Events.Dbt;

public class DbtTaskEventHandler : TaskEventHandler<DbtTaskEvent>, IEventHandler<DbtTaskEvent>
{
    private readonly IDbtService _dbtService;

    public DbtTaskEventHandler(IProcessManager manager,
        SequentialTaskProcessor taskProcessor,
        IDbtService dbtService,
        ChannelWriter<EventStreamResponse> writer,
        ILogger<DbtTaskEventHandler> logger) : base(manager, taskProcessor, writer, logger)
    {
        _dbtService = dbtService;
        CurrentClassName = GetType().Name;
    }

    public async Task HandleAsync(DbtTaskEvent eventModel, CancellationToken ct = new())
    {
        async Task ExecuteTaskAsync()
        {
            LogEventModel("Executing DbtTaskEventHandler", eventModel);
            SetResultStatusInProgress(eventModel);
            var result = await eventModel.HandleAsync(_dbtService);
            await HandleResult(result, eventModel);
        }

        if (!eventModel.RunAsync)
        {
            await ExecuteTaskAsync().ConfigureAwait(false);
            return;
        }

        await EnqueueTaskAsync(eventModel.ProcessReferenceId, ExecuteTaskAsync).ConfigureAwait(false);
    }
}