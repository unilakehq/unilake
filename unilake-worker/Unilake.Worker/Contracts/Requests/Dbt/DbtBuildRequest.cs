using FluentValidation;

namespace Unilake.Worker.Contracts.Requests.Dbt;

public class DbtBuildRequest 
{
    public string TargetName { get; set; }
}

public class DbtBuildRequestValidator : Validator<DbtBuildRequest>
{
    public DbtBuildRequestValidator()
    {
        RuleFor(x => x.TargetName)
            .NotEmpty()
            .WithMessage("Target Name is required")
            .MinimumLength(10)
            .WithMessage("Target Name must be at least 10 characters");
    }
}