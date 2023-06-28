using System.Threading.Channels;
using Unilake.Worker.Contracts.Responses;
using Unilake.Worker.Services;
using Unilake.Worker.Services.Git;

namespace Unilake.Worker.Events.Git;

public class GitTaskEventHandler : TaskEventHandler<GitTaskEvent>, IEventHandler<GitTaskEvent>
{
    private readonly IGitService _gitService;

    public GitTaskEventHandler(IProcessManager manager,
        SequentialTaskProcessor taskProcessor,
        IGitService gitService,
        ChannelWriter<EventStreamResponse> writer,
        ILogger<GitTaskEventHandler> logger) : base(manager, taskProcessor, writer, logger)
    {
        _gitService = gitService;
        CurrentClassName = GetType().Name;
    }

    public async Task HandleAsync(GitTaskEvent eventModel, CancellationToken ct = new())
    {
        async Task ExecuteTaskAsync()
        {
            LogEventModel("Executing GitTaskEventHandler", eventModel);
            SetResultStatusInProgress(eventModel);
            var result = await eventModel.HandleAsync(_gitService);
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