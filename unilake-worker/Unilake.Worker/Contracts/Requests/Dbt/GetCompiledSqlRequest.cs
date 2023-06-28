using FluentValidation;

namespace Unilake.Worker.Contracts.Requests.Dbt;

public class GetCompiledSqlRequest
{
    public string ModelPath { get; set; }
}

public class GetCompiledSqlRequestValidator : Validator<GetCompiledSqlRequest>
{
    public GetCompiledSqlRequestValidator()
    {
        RuleFor(x => x.ModelPath)
            .NotEmpty()
            .WithMessage("Model path is required");
    }
}