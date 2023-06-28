using OneOf;
using OneOf.Types;
using Unilake.Worker.Contracts;
using Unilake.Worker.Contracts.Requests.Git;
using Unilake.Worker.Contracts.Responses.Git;
using Unilake.Worker.Services.Git;

namespace Unilake.Worker.Events.Git.Types;

public class GitCheckoutTaskEvent : GitTaskEvent
{
    public string BranchOrCommit { get; set; }
    public bool CreateBranch { get; set; }
    
    public static implicit operator GitCheckoutTaskEvent(GitCheckoutRequest request) => new()
    {
        BranchOrCommit = request.BranchOrCommit,
        CreateBranch = request.CreateBranch,
    };

    protected override OneOf<Success<IRequestResponse>, Error<string>> Handle(IGitService gitService)
    {
        return gitService.Checkout(BranchOrCommit, CreateBranch)
            .Match<OneOf<Success<IRequestResponse>, Error<string>>>(
                _ => new Success<IRequestResponse>(new GitActionResultResponse()
                {
                    Message = "Successfully checked out branch or commit",
                    ProcessReferenceId = ProcessReferenceId
                }),
                e => new Error<string>(e.Value.Message.FirstToUpper())
            );
    }
}