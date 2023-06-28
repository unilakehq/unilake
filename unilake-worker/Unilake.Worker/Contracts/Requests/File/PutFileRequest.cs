using FluentValidation;

namespace Unilake.Worker.Contracts.Requests.File;

public class PutFileRequest : AsyncRequestOption
{
    public string Path { get; set; }
    public IFormFile Content { get; set; }
    public string GetFullPath() => System.IO.Path.Join(Path, Content.FileName);
}

public class PutFileValidator : Validator<PutFileRequest>
{
    public PutFileValidator()
    {
        RuleFor(x => x.Path)
            .NotEmpty()
            .WithMessage("File path is required.");

        RuleFor(x => x.Content)
            .NotEmpty()
            .WithMessage("File content is required.");

        RuleFor(x => x.Content)
            .Must(x => x.Length < 3e+6)
            .WithMessage("File content must not exceed 3 Megabtytes.");

        RuleFor(x => x.GetFullPath())
            .Must(Path.IsPathFullyQualified)
            .WithMessage("Provided file path must be fully qualified.");
    }
}