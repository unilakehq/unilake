using Unilake.Worker.Contracts.Requests.File;
using Unilake.Worker.Processors.PreProcessor;
using Unilake.Worker.Services.File;

namespace Unilake.Worker.Endpoints.File;

public class GetFile : Endpoint<GetFileRequest>
{
    private readonly IFileService _fileService;

    public GetFile(IFileService fileService)
    {
        _fileService = fileService;
    }

    public override void Configure()
    {
        Get("/file");
        Summary(s =>
        {
            s.Summary = "Get the contents a file";
            s.Description = "Endpoint returns the file content.";
            s.Responses[200] =
                "File content is returned.";
        });
        PreProcessors(new RequestActivityTracker<GetFileRequest>());
    }

    public override async Task HandleAsync(GetFileRequest request, CancellationToken cancellationToken)
    {
        var file = _fileService.GetFile(request.Path);
        await file.Match(
            c => SendStreamAsync(c, cancellation: cancellationToken).ConfigureAwait(false),
            e =>
            {
                Logger.LogError(e, "Could not retrieve file");
                AddError(e.Message);
                return SendErrorsAsync(cancellation: cancellationToken).ConfigureAwait(false);
            }
        );
    }
}