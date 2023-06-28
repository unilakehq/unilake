using OneOf;
using OneOf.Types;
using Unilake.Worker.Contracts;
using Unilake.Worker.Contracts.Requests.File;
using Unilake.Worker.Contracts.Responses.File;
using Unilake.Worker.Services.File;

namespace Unilake.Worker.Events.File.Types;

public class DirectoryDeleteTaskEvent : FileTaskEvent
{
    public string Path { get; set; }
    
    public static implicit operator DirectoryDeleteTaskEvent(DirectoryDeleteRequest request) => new()
    {
        Path = request.Path
    };

    protected override OneOf<Success<IRequestResponse>, Error<string>> Handle(IFileService service)
    {
        return service.DeleteDirectory(Path)
            .Match<OneOf<Success<IRequestResponse>, Error<string>>>(
                _ => new Success<IRequestResponse>(new FileActionResultResponse()
                {
                    Message = "Successfully deleted directory",
                    ProcessReferenceId = ProcessReferenceId
                }),
                e => new Error<string>(e.Message.FirstToUpper())
            );
    }
}