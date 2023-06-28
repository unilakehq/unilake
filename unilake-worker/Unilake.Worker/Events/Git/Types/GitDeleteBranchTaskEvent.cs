using OneOf;
using OneOf.Types;
using Unilake.Worker.Contracts;
using Unilake.Worker.Contracts.Requests.Git;
using Unilake.Worker.Contracts.Responses.Git;
using Unilake.Worker.Services.Git;

namespace Unilake.Worker.Events.Git.Types;

public class GitDeleteBranchTaskEvent : GitTaskEvent
{
    public string BranchName { get; set; }

    public static implicit operator GitDeleteBranchTaskEvent(GitDeleteBranchRequest request) => new()
    {
        BranchName = request.BranchName,
    };

    protected override OneOf<Success<IRequestResponse>, Error<string>> Handle(IGitService gitService)
    {
        return gitService.DeleteBranch(BranchName)
            .Match<OneOf<Success<IRequestResponse>, Error<string>>>(
                _ => new Success<IRequestResponse>(new GitActionResultResponse()
                {
                    Message = "Successfully deleted branch",
                    ProcessReferenceId = ProcessReferenceId
                }),
                e => new Error<string>(e.Value.Message.FirstToUpper())
            );
    }
}
