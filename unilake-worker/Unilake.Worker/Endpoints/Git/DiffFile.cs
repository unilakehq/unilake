using Unilake.Worker.Contracts.Requests.Git;
using Unilake.Worker.Contracts.Responses.Git;
using Unilake.Worker.Mappers.Git;
using Unilake.Worker.Processors.PreProcessor;
using Unilake.Worker.Services.Git;

namespace Unilake.Worker.Endpoints.Git;

public class DiffFile : Endpoint<GitDiffFileRequest, GitDiffFileResponse[], DiffFileMapper>
{
    private readonly IGitService _gitService;

    public DiffFile(IGitService gitService)
    {
        _gitService = gitService;
    }

    public override void Configure()
    {
        Get("/git/branch/diff/file");
        Summary(s =>
        {
            s.Summary = "Return a list of file changes.";
            s.Description = "Compare different files between two branches and return the changes made to these files.";
            s.Responses[200] = "List of changes.";
        });
        PreProcessors(new RequestActivityTracker<GitDiffFileRequest>());
    }

    public override async Task HandleAsync(GitDiffFileRequest request, CancellationToken cancellationToken)
    {
        await _gitService.GetFileDiff(request.SourceBranch, request.TargetBranch, request.FilePaths).Match(
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