using OneOf;
using OneOf.Types;
using Unilake.Worker.Contracts;
using Unilake.Worker.Contracts.Requests.Git;
using Unilake.Worker.Services.Git;

namespace Unilake.Worker.Events.Git.Types;

public class GitBranchMergeTaskEvent : GitTaskEvent
{
    public string TargetBranch { get; set; }

    public static implicit operator GitBranchMergeTaskEvent(GitBranchMergeRequest request) => new()
    {
        TargetBranch = request.TargetBranch
    };

    protected override OneOf<Success<IRequestResponse>, Error<string>> Handle(IGitService gitService)
    {
        throw new NotImplementedException();
    }
}