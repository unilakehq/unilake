using FluentValidation;

namespace Unilake.Worker.Contracts.Requests.Git;

public class GitCommitRequest : AsyncRequestOption
{
    public string Message { get; set; }
}

public class GitCommithValidator : Validator<GitCommitRequest>
{
    public GitCommithValidator()
    {
        RuleFor(x => x.Message)
           .NotEmpty()
           .WithMessage("Branch must be specified");

    }
}