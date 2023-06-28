using FluentValidation;

namespace Unilake.Worker.Contracts.Requests.Dbt;

public class CompileModelRequest
{
    public string ModelPath { get; set; }

    public string ModelType { get; set; }
}

public class CompileModelRequestValidator : Validator<CompileModelRequest>
{
    public CompileModelRequestValidator()
    {
        RuleFor(x => x.ModelPath)
            .NotEmpty()
            .WithMessage("Model path is required");
        
        RuleFor(x => x.ModelType)
            .NotEmpty()
            .WithMessage("Model type is required");
    }
}