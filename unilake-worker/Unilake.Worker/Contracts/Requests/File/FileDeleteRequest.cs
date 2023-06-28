using FluentValidation;

namespace Unilake.Worker.Contracts.Requests.File;

public class FileDeleteRequest : AsyncRequestOption
{
    public string Path { get; set; }
}

public class FileDeleteValidator : Validator<FileDeleteRequest>
{
    public FileDeleteValidator()
    {
        RuleFor(x => x.Path)
            .NotEmpty()
            .WithMessage("File path is required.");
    }
}