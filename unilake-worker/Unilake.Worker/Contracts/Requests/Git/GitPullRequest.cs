using FluentValidation;

namespace Unilake.Worker.Contracts.Requests.Git;

public class GitPullRequest : AsyncRequestOption
{
    public string Remote { get; set; }
    public string Branch { get; set; }
}

public class GitPullValidator : Validator<GitPullRequest>
{
    public GitPullValidator()
    {
        RuleFor(x => x.Remote)
           .NotEmpty()
           .WithMessage("Remote must be specified");

        RuleFor(x => x.Branch)
           .NotEmpty()
           .WithMessage("Branch must be specified");

    }
}