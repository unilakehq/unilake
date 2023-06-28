using OneOf;
using OneOf.Types;
using Unilake.Worker.Contracts;
using Unilake.Worker.Contracts.Requests.Dbt;
using Unilake.Worker.Contracts.Responses.Dbt;
using Unilake.Worker.Models.Dbt;
using Unilake.Worker.Services.Dbt;

namespace Unilake.Worker.Events.Dbt.Types;

public class BuildModelTaskEvent : DbtTaskEvent
{
    public string ModelPath { get; set; }
    public string ModelName { get; set; }
    public RunModelType ModelType { get; set; }
    
    public static implicit operator BuildModelTaskEvent(BuildModelRequest request) => new()
    {
        ModelName = request.ModelName,
        ModelPath = request.ModelPath,
        ModelType = Enum.Parse<RunModelType>(request.ModelType),
    };

    public override async Task<OneOf<Success<IRequestResponse>, Error<string>>> HandleAsync(IDbtService service)
    {
        return (await service.BuildModelAsync(null, new Uri(ModelPath), ModelType, CancellationToken.None))
            .Match<OneOf<Success<IRequestResponse>, Error<string>>>(
                _ => new Success<IRequestResponse>(new DbtActionResultResponse()
                {
                    Message = "Successfully built DBT model",
                    ProcessReferenceId = ProcessReferenceId
                }),
                e => new Error<string>(e.Value)
            );
    }
}