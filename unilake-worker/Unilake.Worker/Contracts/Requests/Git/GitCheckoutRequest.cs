using FluentValidation;

namespace Unilake.Worker.Contracts.Requests.Git;

public class GitCheckoutRequest : AsyncRequestOption
{
    public string BranchOrCommit { get; init; }
    public bool CreateBranch { get; init; } = false;
}

public class GitCheckoutValidator : Validator<GitCheckoutRequest>
{
    public GitCheckoutValidator()
    {
        RuleFor(x => x.BranchOrCommit)
           .NotEmpty()
           .WithMessage("Either branch name or commit hash must be specified");
    }
}