using OneOf;
using OneOf.Types;
using Unilake.Worker.Contracts;
using Unilake.Worker.Models.Dbt;
using Unilake.Worker.Services.Dbt.Command;

namespace Unilake.Worker.Services.Dbt.Manifest;

// TODO: workspace folder is a vscdode thing, need to check what this is and implement for I think the current workfolder
// Instead of workspace folders we have dataproducts, so each dataproduct has its own folder and dbt project (multiple dbt projects in one domain)

public class DbtProjectContainer : IDbtService
{
    private readonly List<DbtDataProductFolder> _dbtDataProductFolders = new();
    private readonly DbtClient _dbtClient;

    public DbtProjectContainer(
        IConfiguration configuration,
        DbtClient dbtClient)
    {
        _dbtClient = dbtClient;
    }
    public Task InitializeDBTProjects()
    {
        var locations = new List<DataProduct>();
        foreach (var loc in locations)
            RegisterDataProduct(loc);

        return Task.CompletedTask;
    }

    public Task<OneOf<Success, Error<string>>> RunModelTestAsync(IRequestResponse request, Uri modelPath,
        string modelName,
        CancellationToken cancellationToken) => FindDbtProject(modelPath).Match(
        p => p.RunModelTest(request.ProcessReferenceId, modelName, cancellationToken),
        _ => Task.FromResult<OneOf<Success, Error<string>>>(new Error<string>("Could not find Uri")));

    public Task<OneOf<Success, Error<string>>> RunTestAsync(IRequestResponse request, Uri modelPath, string testName,
        CancellationToken cancellationToken) => FindDbtProject(modelPath).Match(
        p => p.RunTest(request.ProcessReferenceId, testName, cancellationToken),
        _ => Task.FromResult<OneOf<Success, Error<string>>>(new Error<string>("Could not find Uri")));

    public Task<OneOf<Success, Error<string>>> CompileModelAsync(IRequestResponse request, Uri modelPath,
        RunModelType modelType, CancellationToken cancellationToken) => FindDbtProject(modelPath).Match(
        p => p.CompileModel(request.ProcessReferenceId, CreateModelParams(modelPath, modelType), cancellationToken),
        _ => Task.FromResult<OneOf<Success, Error<string>>>(new Error<string>("Could not find Uri"))
    );

    public OneOf<Success<CompilationResult>, Error<string>> CompileQuery(Uri modelPath, string query) => FindDbtProject(modelPath).Match(
        p =>
        {
            throw new NotImplementedException();
            // var result = p.CompileQuery(query);
            // return result.IsT0 ? result.AsT0 : new Error<string>($"Could not compile query {query}, due to error: {result.AsT1.Value}");
        },
        _ => new Error<string>("Could not find Uri")
    );

    public OneOf<Success<string>, Error<string>> GetRunSql(Uri modelPath) => FindDbtProject(modelPath).Match(
        p => p.GetRunSqlPath(modelPath),
        _ => new Error<string>("Could not find Uri")
    );

    public OneOf<Success<string>, Error<string>> GetCompiledSql(Uri modelPath) => FindDbtProject(modelPath).Match(
        p => p.GetCompiledSqlPath(modelPath),
        _ => new Error<string>("Could not find Uri")
    );

    public Task<OneOf<Success, Error<string>>> BuildModelAsync(IRequestResponse request, Uri modelPath,
        RunModelType modelType, CancellationToken cancellationToken) => FindDbtProject(modelPath).Match(
        p => p.BuildModel(request.ProcessReferenceId, CreateModelParams(modelPath, modelType), cancellationToken),
        _ => Task.FromResult<OneOf<Success, Error<string>>>(new Error<string>("Could not find Uri"))
    );

    public Task<OneOf<Success, Error<string>>> RunModelAsync(IRequestResponse request, Uri modelPath,
        RunModelType modelType,
        CancellationToken cancellationToken) => FindDbtProject(modelPath).Match(
        p => p.RunModel(request.ProcessReferenceId, CreateModelParams(modelPath, modelType), cancellationToken),
        _ => Task.FromResult<OneOf<Success, Error<string>>>(new Error<string>("Could not find Uri"))
    );

    public OneOf<Success<string>, Error<string>> GetProjectRootPath(Uri modelPath) => FindDbtProject(modelPath).Match<OneOf<Success<string>, Error<string>>>(
        p => new Success<string>(p.ProjectRoot.ToString()),
        _ => new Error<string>("Could not find Uri")
    );

    private OneOf<DbtDataProductFolder, None> FindDbtDataProductFolder(Uri uri)
    {
        var found = _dbtDataProductFolders.Find(pr => pr.DbtProject.ProjectRoot.IsBaseOf(uri));
        return found != null ? found : new None();
    }

    private OneOf<DbtProject, None> FindDbtProject(Uri uri) => FindDbtDataProductFolder(uri)
        .Match<OneOf<DbtProject, None>>(
            f => f.DbtProject,
            n => n
        );

    private RunModelParams CreateModelParams(Uri modelPath, RunModelType? type = null) => new()
    {
        ModelName = Path.GetFileNameWithoutExtension(modelPath.LocalPath),
        PlusOperatorLeft = type == RunModelType.Parents ? "+" : "",
        PlusOperatorRight = type == RunModelType.Children ? "+" : ""
    };

    private void RegisterDataProduct(DataProduct dataProduct)
    {
        var dbtProjectWorkspaceFolder = new DbtDataProductFolder(dataProduct.RootPath, _dbtClient);
        _dbtDataProductFolders.Add(dbtProjectWorkspaceFolder);
        dbtProjectWorkspaceFolder.DiscoverProject();
    }

    private void UnregisterDataProduct(DataProduct dataProduct)
    {
        throw new NotImplementedException();
    }
}