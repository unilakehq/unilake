using FluentValidation;

namespace Unilake.Worker.Contracts.Requests.Git;

public class GitCreateBranchRequest : AsyncRequestOption
{
    public string BranchName { get; set; }
}

public class GitCreateBranchValidator : Validator<GitCreateBranchRequest>
{
    public GitCreateBranchValidator()
    {
        RuleFor(x => x.BranchName)
           .NotEmpty()
           .WithMessage("Branch must be specified");

    }
}