using Unilake.Worker.Contracts.Requests.File;
using Unilake.Worker.Contracts.Responses.File;
using Unilake.Worker.Mappers.File;
using Unilake.Worker.Processors.PreProcessor;
using Unilake.Worker.Services.File;

namespace Unilake.Worker.Endpoints.File;

public class DirectoryList : Endpoint<DirectoryListRequest, DirectoryListResponse, DirectoryContentMapper>
{
    private readonly IFileService _fileService;

    public DirectoryList(IFileService fileService)
    {
        _fileService = fileService;
    }

    public override void Configure()
    {
        Get("/file/list");
        Summary(s =>
        {
            s.Summary = "List all files in a directory";
            s.Description = "List all files and subdirectories present in the given path.";
            s.Responses[200] =
                "Overview of files and directories in the given path.";
        });
        PreProcessors(new RequestActivityTracker<DirectoryListRequest>());
    }

    public override async Task HandleAsync(DirectoryListRequest request, CancellationToken cancellationToken)
    {
        await _fileService.GetDirectoryContent(request.Path).Match(
            o => SendAsync(Map.FromEntity(o), cancellation: cancellationToken).ConfigureAwait(false),
            e =>
            {
                Logger.LogError(e, CommonMessages.AnErrorOccuredWhileRetrievingTheEvent);
                AddError(e.Message);
                return SendErrorsAsync(cancellation: cancellationToken).ConfigureAwait(false);
            }
        );
    }
}