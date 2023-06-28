using FluentValidation;

namespace Unilake.Worker.Contracts.Requests.Dbt;

public class CompileQueryRequest
{
    public string ModelPath { get; set; }

    public string Query { get; set; }
}

public class CompileQueryRequestValidator : Validator<CompileQueryRequest>
{
    public CompileQueryRequestValidator()
    {
        RuleFor(x => x.ModelPath)
            .NotEmpty()
            .WithMessage("Model path is required");
        
        RuleFor(x => x.Query)
            .NotEmpty()
            .WithMessage("Query is required");
    }
}