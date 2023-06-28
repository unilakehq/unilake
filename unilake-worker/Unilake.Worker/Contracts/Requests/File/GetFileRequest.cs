using FluentValidation;

namespace Unilake.Worker.Contracts.Requests.File;

public class GetFileRequest : AsyncRequestOption
{
    public string Path { get; set; }
}

public class GetFileValidator : Validator<GetFileRequest>
{
    public GetFileValidator()
    {
        RuleFor(x => x.Path)
            .NotEmpty()
            .WithMessage("File path is required.");
    }
}