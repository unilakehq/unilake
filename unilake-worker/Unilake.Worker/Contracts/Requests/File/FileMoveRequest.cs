using FluentValidation;

namespace Unilake.Worker.Contracts.Requests.File;

public class FileMoveRequest : AsyncRequestOption
{
    public string SourcePath { get; set; }
    public string TargetPath { get; set; }
}

public class FileMoveValidator : Validator<FileMoveRequest>
{
    public FileMoveValidator()
    {
        RuleFor(x => x.SourcePath)
            .NotEmpty()
            .WithMessage("File source path is required.");
        RuleFor(x => x.TargetPath)
            .NotEmpty()
            .WithMessage("File target path is required.");
    }
}