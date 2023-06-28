using FluentValidation;

namespace Unilake.Worker.Contracts.Requests.Git;

public class GitCloneRequest : AsyncRequestOption
{
    public string RepoUrl { get; init; }
    public string Branch { get; set; }
}

public class GitCloneValidator : Validator<GitCloneRequest>
{
    public GitCloneValidator()
    {
        RuleFor(x => x.RepoUrl)
           .NotEmpty()
           .WithMessage("Repository URL is required");
    }
}