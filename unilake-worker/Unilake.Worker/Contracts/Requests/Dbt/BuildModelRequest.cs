using FluentValidation;
using Unilake.Worker.Models.Dbt;

namespace Unilake.Worker.Contracts.Requests.Dbt;

public class BuildModelRequest : AsyncRequestOption
{
    public string ModelPath { get; set; }

    public string ModelName { get; set; }
    public string ModelType { get; set; }
}

public class BuildModelRequestValidator : Validator<BuildModelRequest>
{
    public BuildModelRequestValidator()
    {
        RuleFor(x => x.ModelPath)
            .NotEmpty()
            .WithMessage("Model path is required");
        
        RuleFor(x => x.ModelName)
            .NotEmpty()
            .WithMessage("Model name is required");

        RuleFor(x => x.ModelType)
            .IsEnumName(typeof(RunModelType), caseSensitive: false)
            .WithMessage("Model type is should any of the following values: 'Parents', 'Children', 'Test', 'Snapshot'");
    }
}