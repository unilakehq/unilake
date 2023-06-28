using FluentValidation;

namespace Unilake.Worker.Contracts.Requests.Dbt;

public class RunTestRequest
{
    public string ModelPath { get; set; }

    public string TestName { get; set; }
}

public class RunTestRequestValidator : Validator<RunTestRequest>
{
    public RunTestRequestValidator()
    {
        RuleFor(x => x.ModelPath)
            .NotEmpty()
            .WithMessage("Model path is required");
        
        RuleFor(x => x.TestName)
            .NotEmpty()
            .WithMessage("Test name is required");
    }
}