using OneOf;
using OneOf.Types;
using Unilake.Worker.Contracts;
using Unilake.Worker.Contracts.Requests.Git;
using Unilake.Worker.Contracts.Responses.Git;
using Unilake.Worker.Services.Git;

namespace Unilake.Worker.Events.Git.Types;

public class GitCloneTaskEvent : GitTaskEvent
{
    public string RepoUrl { get; set; }
    public string Branch { get; set; }

    public static implicit operator GitCloneTaskEvent(GitCloneRequest request) => new ()
    {
        Branch = request.Branch,
        RepoUrl = request.RepoUrl
    };

    protected override OneOf<Success<IRequestResponse>, Error<string>> Handle(IGitService gitService)
    {
        return gitService.Clone(RepoUrl, Environment.CurrentDirectory)
            .Match<OneOf<Success<IRequestResponse>, Error<string>>>(
                _ => new Success<IRequestResponse>(new GitActionResultResponse()
                {
                    Message = "Successfully cloned repository",
                    ProcessReferenceId = ProcessReferenceId
                }),
                e => new Error<string>(e.Value.Message.FirstToUpper())
            );
    }
}