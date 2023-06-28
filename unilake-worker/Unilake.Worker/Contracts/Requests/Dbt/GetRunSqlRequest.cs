using FluentValidation;

namespace Unilake.Worker.Contracts.Requests.Dbt;

public class GetRunSqlRequest
{
    public string ModelPath { get; set; }
}

public class GetRunSqlRequestValidator : Validator<GetRunSqlRequest>
{
    public GetRunSqlRequestValidator()
    {
        RuleFor(x => x.ModelPath)
            .NotEmpty()
            .WithMessage("Model path is required");
    }
}