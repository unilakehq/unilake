using FluentValidation;

namespace Unilake.Worker.Contracts.Requests.Git;

public class GitBranchMergeRequest : AsyncRequestOption
{
    public string TargetBranch { get; set; }
}

public class GitBranchMergeValidator : Validator<GitBranchMergeRequest>
{
    public GitBranchMergeValidator()
    {
        RuleFor(x => x.TargetBranch)
            .NotEmpty()
            .WithMessage("Target branch can't be empty");
    }
}