using FluentValidation;

namespace Unilake.Worker.Contracts.Requests.File;

public class DirectoryCreateRequest : AsyncRequestOption
{
    public string Path { get; set; }
}

public class DirectoryCreateValidator : Validator<DirectoryCreateRequest>
{
    public DirectoryCreateValidator()
    {
        RuleFor(x => x.Path)
            .NotEmpty()
            .WithMessage("Path is required.");
    }
}