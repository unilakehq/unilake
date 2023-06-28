using System.Threading.Channels;
using Unilake.Worker.Contracts.Responses;
using Unilake.Worker.Services;
using Unilake.Worker.Services.File;

namespace Unilake.Worker.Events.File;

public class FileTaskEventHandler : TaskEventHandler<FileTaskEvent>, IEventHandler<FileTaskEvent>
{
    private readonly IFileService _fileService;

    public FileTaskEventHandler(IProcessManager manager,
        SequentialTaskProcessor taskProcessor,
        IFileService fileService,
        ChannelWriter<EventStreamResponse> writer,
        ILogger<FileTaskEventHandler> logger) : base(manager, taskProcessor, writer, logger)
    {
        _fileService = fileService;
        CurrentClassName = GetType().Name;
    }

    public async Task HandleAsync(FileTaskEvent eventModel, CancellationToken ct = new())
    {
        async Task ExecuteTaskAsync()
        {
            LogEventModel("Executing FileTaskEventHandler", eventModel);
            SetResultStatusInProgress(eventModel);
            var result = await eventModel.HandleAsync(_fileService);
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