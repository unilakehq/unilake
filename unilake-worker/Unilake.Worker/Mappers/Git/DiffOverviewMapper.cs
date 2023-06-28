using Unilake.Worker.Contracts.Requests.Git;
using Unilake.Worker.Contracts.Responses.Git;
using Unilake.Worker.Models.Git;

namespace Unilake.Worker.Mappers.Git;

public class DiffOverviewMapper : Mapper<GitDiffOverviewRequest, GitDiffOverviewResponse[], GitDiff[]>
{
    public override GitDiffOverviewResponse[] FromEntity(GitDiff[] e) => e.Select(n => new GitDiffOverviewResponse
    {
        Kind = n.Status.ToString(),
        ObjectId = n.Oid.ToString(),
        FilePath = n.Path
    }).ToArray();
}