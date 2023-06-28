using FluentValidation;

namespace Unilake.Worker.Contracts.Requests.Git;

public class GitDeleteBranchRequest : AsyncRequestOption
{
    public string BranchName { get; set; }
}

public class GitDeleteBranchValidator : Validator<GitDeleteBranchRequest>
{
    public GitDeleteBranchValidator()
    {
        RuleFor(x => x.BranchName)
           .NotEmpty()
           .WithMessage("Branch must be specified");

    }
}