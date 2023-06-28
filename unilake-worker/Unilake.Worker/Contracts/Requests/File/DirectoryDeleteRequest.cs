using FluentValidation;

namespace Unilake.Worker.Contracts.Requests.File;

public class DirectoryDeleteRequest : AsyncRequestOption
{
    public string Path { get; set; }
}

public class DirectoryDeleteValidator : Validator<DirectoryDeleteRequest>
{
    public DirectoryDeleteValidator()
    {
        RuleFor(x => x.Path)
            .NotEmpty()
            .WithMessage("Path is required.");
    }
}