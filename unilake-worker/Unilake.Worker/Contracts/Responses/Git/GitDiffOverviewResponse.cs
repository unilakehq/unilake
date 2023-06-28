namespace Unilake.Worker.Contracts.Responses.Git;

public class GitDiffOverviewResponse
{
    public string ObjectId { get; set; }
    public string Kind { get; set; }
    public string FilePath { get; set; }
}