using FluentValidation;

namespace Unilake.Worker.Contracts.Requests.Git;

public class GitDiffOverviewRequest
{
    public string SourceBranch { get; set; }
    public string TargetBranch { get; set; }
}

public class GitDiffOverviewValidator : Validator<GitDiffOverviewRequest>
{
    public GitDiffOverviewValidator()
    {
        RuleFor(x => x.SourceBranch)
            .NotEmpty()
            .WithMessage("Source branch must be specified");
        RuleFor(x => x.TargetBranch)
            .NotEmpty()
            .WithMessage("Target branch must be specified");
    }
}