using OneOf;
using OneOf.Types;
using Unilake.Worker.Contracts;
using Unilake.Worker.Models.Dbt;
using Unilake.Worker.Services.Dbt.Manifest;

namespace Unilake.Worker.Services.Dbt;

public interface IDbtService
{
    Task<OneOf<Success, Error<string>>> RunModelTestAsync(IRequestResponse request, Uri modelPath, string modelName,
        CancellationToken cancellationToken);

    Task<OneOf<Success, Error<string>>> RunTestAsync(IRequestResponse request, Uri modelPath, string testName,
        CancellationToken cancellationToken);

    Task<OneOf<Success, Error<string>>> CompileModelAsync(IRequestResponse request, Uri modelPath,
        RunModelType modelType, CancellationToken cancellationToken);

    OneOf<Success<CompilationResult>, Error<string>> CompileQuery(Uri modelPath, string query);
    OneOf<Success<string>, Error<string>> GetRunSql(Uri modelPath);
    OneOf<Success<string>, Error<string>> GetCompiledSql(Uri modelPath);

    Task<OneOf<Success, Error<string>>> BuildModelAsync(IRequestResponse request, Uri modelPath, RunModelType modelType,
        CancellationToken cancellationToken);

    Task<OneOf<Success, Error<string>>> RunModelAsync(IRequestResponse request, Uri modelPath, RunModelType modelType,
        CancellationToken cancellationToken);

    OneOf<Success<string>, Error<string>> GetProjectRootPath(Uri modelPath);
}