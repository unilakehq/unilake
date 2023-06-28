using OneOf;
using OneOf.Types;
using Unilake.Worker.Contracts;
using Unilake.Worker.Contracts.Requests.File;
using Unilake.Worker.Contracts.Responses.File;
using Unilake.Worker.Services.File;

namespace Unilake.Worker.Events.File.Types;

public class FileMoveTaskEvent : FileTaskEvent
{
    public string SourcePath { get; set; }
    
    public string TargetPath { get; set; }
    
    public static implicit operator FileMoveTaskEvent(FileMoveRequest request) => new()
    {
        SourcePath = request.SourcePath,
        TargetPath = request.TargetPath
    };

    protected override OneOf<Success<IRequestResponse>, Error<string>> Handle(IFileService service)
    {
        return service.MoveFile(SourcePath, TargetPath)
            .Match<OneOf<Success<IRequestResponse>, Error<string>>>(
                _ => new Success<IRequestResponse>(new FileActionResultResponse()
                {
                    Message = "Successfully moved file",
                    ProcessReferenceId = ProcessReferenceId
                }),
                e => new Error<string>(e.Message.FirstToUpper())
            );
    }
}