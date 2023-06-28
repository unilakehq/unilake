using FluentValidation;

namespace Unilake.Worker.Contracts.Requests.File;

public class DirectoryMoveRequest : AsyncRequestOption
{
    public string SourcePath { get; set; }
    public string TargetPath { get; set; }
}

public class DirectoryMoveValidator : Validator<DirectoryMoveRequest>
{
    public DirectoryMoveValidator()
    {
        RuleFor(x => x.SourcePath)
            .NotEmpty()
            .WithMessage("SourcePath is required.");
        
        RuleFor(x => x.TargetPath)
            .NotEmpty()
            .WithMessage("TargetPath is required.");
    }
}