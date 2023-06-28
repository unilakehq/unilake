using FluentValidation;

namespace Unilake.Worker.Contracts.Requests.Git;

public class GitPushRequest : AsyncRequestOption
{
    public string Remote { get; set; }
    public string Branch { get; set; }
}

public class GitPushValidator : Validator<GitPushRequest>
{
    public GitPushValidator()
    {
        RuleFor(x => x.Remote)
           .NotEmpty()
           .WithMessage("Remote must be specified");

        RuleFor(x => x.Branch)
           .NotEmpty()
           .WithMessage("Branch must be specified");

    }
}