using Unilake.Worker.Contracts.Requests.File;
using Unilake.Worker.Processors.PreProcessor;
using Unilake.Worker.Services.File;

namespace Unilake.Worker.Endpoints.File;

public class PutFile : Endpoint<PutFileRequest>
{
    private readonly IFileService _fileService;

    public PutFile(IFileService fileService)
    {
        _fileService = fileService;
    }

    public override void Configure()
    {
        Put("/file");
        AllowFileUploads();
        Summary(s =>
        {
            s.Summary = "Send the contents a file";
            s.Description = "Endpoint receives file content and saves this to a local location.";
            s.Responses[200] =
                "File content is saved successfully.";
        });
        PreProcessors(new RequestActivityTracker<PutFileRequest>());
    }

    public override async Task HandleAsync(PutFileRequest request, CancellationToken cancellationToken)
    {
        var file = _fileService.PutFile(request.GetFullPath(), request.Content.OpenReadStream());
        await file.Match(
            _ => SendOkAsync(cancellationToken).ConfigureAwait(false),
            e =>
            {
                Logger.LogError(e, "Could not save file");
                AddError(e.Message);
                return SendErrorsAsync(cancellation: cancellationToken).ConfigureAwait(false);
            }
        );
    }
}