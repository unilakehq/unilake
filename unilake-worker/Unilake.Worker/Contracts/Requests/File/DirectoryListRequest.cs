using FluentValidation;

namespace Unilake.Worker.Contracts.Requests.File;

public class DirectoryListRequest
{
    public string Path { get; set; }
}

public class DirectoryListValidator : Validator<DirectoryListRequest>
{
    public DirectoryListValidator()
    {
        RuleFor(x => x.Path)
            .NotEmpty()
            .WithMessage("Path is required.");
    }
}