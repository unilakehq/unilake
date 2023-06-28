using OneOf;
using OneOf.Types;
using Unilake.Worker.Contracts;
using Unilake.Worker.Contracts.Responses.Git;
using Unilake.Worker.Services.Git;

namespace Unilake.Worker.Events.Git.Types;

public class GitAbortMergeTaskEvent : GitTaskEvent
{
    protected override OneOf<Success<IRequestResponse>, Error<string>> Handle(IGitService gitService)
    {
        return gitService.AbortMerge()
            .Match<OneOf<Success<IRequestResponse>, Error<string>>>(
                _ => new Success<IRequestResponse>(new GitActionResultResponse()
                {
                    Message = "Successfully aborted merge",
                    ProcessReferenceId = ProcessReferenceId
                }),
                e => new Error<string>(e.Value.Message.FirstToUpper())
            );
    }
}