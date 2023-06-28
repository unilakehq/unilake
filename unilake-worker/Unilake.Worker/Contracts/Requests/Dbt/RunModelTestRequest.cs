using FluentValidation;

namespace Unilake.Worker.Contracts.Requests.Dbt;

public class RunModelTestRequest
{
    public string ModelPath { get; set; }

    public string ModelName { get; set; }
}

public class RunModelTestRequestValidator : Validator<RunModelTestRequest>
{
    public RunModelTestRequestValidator()
    {
        RuleFor(x => x.ModelPath)
            .NotEmpty()
            .WithMessage("Model path is required");
        
        RuleFor(x => x.ModelName)
            .NotEmpty()
            .WithMessage("Test name is required");
    }
}