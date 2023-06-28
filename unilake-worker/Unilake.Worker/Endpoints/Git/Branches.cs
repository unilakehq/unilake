using Unilake.Worker.Contracts.Responses.Git;
using Unilake.Worker.Services.Git;

namespace Unilake.Worker.Endpoints.Git;

public class Branches : EndpointWithoutRequest<GitBranchesResponse[]>
{
    private readonly IGitService _gitService;

    public Branches(IGitService gitService)
    {
        _gitService = gitService;
    }

    public override void Configure()
    {
        Get("/git/branch");
        Summary(s =>
        {
            s.Summary = "Return a list of branches";
            s.Description =
                "Given the current repository, returns the list of branches. This endpoint does not do a git fetch before returning the list of branches";
            s.Responses[200] = "List of branches.";
        });
    }

    public override async Task HandleAsync(CancellationToken cancellationToken)
    {
        var currentBranch = _gitService.ActiveBranch().Match(
            success => success.Value,
            error =>
            {
                Logger.LogError(error.Value, "Could not fetch currently active branch");
                AddError("Failed to fetch currently active branch");
                return "";
            }
        );

        if (string.IsNullOrWhiteSpace(currentBranch))
        {
            await SendErrorsAsync().ConfigureAwait(false);
            return;
        }

        await _gitService.Branches().Match(
            success => SendAsync(success.Value.Select((name, _) => new GitBranchesResponse
            {
                Name = name,
                IsActive = name == currentBranch
            }).ToArray(), cancellation: cancellationToken).ConfigureAwait(false),
            error =>
            {
                Logger.LogError(error.Value, "Could not fetch branches");
                AddError("Failed to fetch branches");
                return SendErrorsAsync(cancellation: cancellationToken).ConfigureAwait(false);
            }
        );
    }
}