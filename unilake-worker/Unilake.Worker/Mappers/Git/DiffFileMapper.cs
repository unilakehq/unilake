using Unilake.Worker.Contracts.Requests.Git;
using Unilake.Worker.Contracts.Responses.Git;
using Unilake.Worker.Models.Git;

namespace Unilake.Worker.Mappers.Git;

public class DiffFileMapper : Mapper<GitDiffFileRequest, GitDiffFileResponse[], GitFileDiff[]>
{
    public override GitDiffFileResponse[] FromEntity(GitFileDiff[] files) => files.Select(f => new GitDiffFileResponse()
    {
        Kind = f.Kind.ToString(),
        SourceContent = f.SourceContent,
        TargetContent = f.TargetContent,
        OldPath = f.OldPath,
        NewPath = f.NewPath,
        ObjectId = f.ObjectId
    }).ToArray();
}