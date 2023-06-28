using OneOf;
using OneOf.Types;
using Unilake.Worker.Contracts;
using Unilake.Worker.Contracts.Requests.Git;
using Unilake.Worker.Contracts.Responses.Git;
using Unilake.Worker.Services.Git;

namespace Unilake.Worker.Events.Git.Types;

public class GitRevertTaskEvent : GitTaskEvent
{
    public string[] Files { get; set; }
    
    public static implicit operator GitRevertTaskEvent(GitRevertRequest request) => new()
    {
        Files = request.Files
    };

    protected override OneOf<Success<IRequestResponse>, Error<string>> Handle(IGitService gitService)
    {
        return gitService.Revert(Files)
            .Match<OneOf<Success<IRequestResponse>, Error<string>>>(
                _ => new Success<IRequestResponse>(new GitActionResultResponse()
                {
                    Message = "Successfully reverted local changes",
                    ProcessReferenceId = ProcessReferenceId
                }),
                e => new Error<string>(e.Value.Message.FirstToUpper())
            );
    }
}
