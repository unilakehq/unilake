using FluentValidation;

namespace Unilake.Worker.Contracts.Requests.Git;

public class GitFetchRequest : AsyncRequestOption
{
    public string Remote { get; set; }
}

public class GitFetchValidator : Validator<GitFetchRequest>
{
    public GitFetchValidator()
    {
        RuleFor(x => x.Remote)
           .NotEmpty()
           .WithMessage("Remote must be specified");
    }
}
