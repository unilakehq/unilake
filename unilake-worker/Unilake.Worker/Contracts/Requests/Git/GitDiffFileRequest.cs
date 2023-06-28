using FluentValidation;

namespace Unilake.Worker.Contracts.Requests.Git;

public class GitDiffFileRequest
{
    public string[] FilePaths { get; set; }
    public string SourceBranch { get; set; }
    public string TargetBranch { get; set; }
}

public class GitDiffFileValidator : Validator<GitDiffFileRequest>
{
    public GitDiffFileValidator()
    {
        RuleFor(x => x.SourceBranch)
            .NotEmpty()
            .WithMessage("Source branch must be specified");
        RuleFor(x => x.TargetBranch)
            .NotEmpty()
            .WithMessage("Target branch must be specified");
    }
}