using LibGit2Sharp;

namespace Unilake.Worker.Models.Git;

public class GitFileDiff
{
    public string ObjectId { get; set; }
    public ChangeKind Kind { get; set; }
    public string SourceContent { get; set; }
    public string TargetContent { get; set; }
    public string OldPath { get; set; }
    public string NewPath { get; set; }
}