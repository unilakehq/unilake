using Unilake.Worker.Contracts.Requests.Git;
using Unilake.Worker.Contracts.Responses.Git;
using Unilake.Worker.Mappers.Git;
using Unilake.Worker.Processors.PreProcessor;
using Unilake.Worker.Services.Git;

namespace Unilake.Worker.Endpoints.Git;

public class DiffOverview : Endpoint<GitDiffOverviewRequest, GitDiffOverviewResponse[], DiffOverviewMapper>
{
    private readonly IGitService _gitService;

    public DiffOverview(IGitService gitService)
    {
        _gitService = gitService;
    }

    public override void Configure()
    {
        Get("/git/branch/diff");
        Summary(s =>
        {
            s.Summary = "Return a list of changes between two branches.";
            s.Description = "Given the current branch, returns a list of changes between branches.";
            s.Responses[200] = "List of changes.";
        });
        PreProcessors(new RequestActivityTracker<GitDiffOverviewRequest>());
    }

    public override async Task HandleAsync(GitDiffOverviewRequest overviewRequest, CancellationToken cancellationToken)
    {
        await _gitService.GetDiff(overviewRequest.SourceBranch, overviewRequest.TargetBranch).Match(
            success => SendAsync(Map.FromEntity(success.Value), cancellation: cancellationToken).ConfigureAwait(false),
            error =>
            {
                Logger.LogError(error.Value, "Could not fetch branches");
                AddError("Failed to fetch branches");
                return SendErrorsAsync(cancellation: cancellationToken).ConfigureAwait(false);
            }
        );
    }
}