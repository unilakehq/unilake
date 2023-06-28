namespace Unilake.Worker.Contracts.Responses.Git;

public class GitDiffFileResponse
{
    public string ObjectId { get; set; }
    public string OldPath { get; set; }
    public string NewPath { get; set; }
    public string Kind { get; set; }
    public string SourceContent { get; set; }
    public string TargetContent { get; set; }
}