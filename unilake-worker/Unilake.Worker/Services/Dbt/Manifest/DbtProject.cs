using System.Globalization;
using OneOf;
using OneOf.Types;
using Unilake.Worker.Services.Dbt.Command;
using YamlDotNet.Serialization.NamingConventions;

namespace Unilake.Worker.Services.Dbt.Manifest;

public class DbtProject
{
    // public static readonly string RUN_RESULTS_FILE = "run_results.json";
    public const string DbtProjectFile = "dbt_project.yml";
    public static readonly string[] DbtModules = { "dbt_modules", "dbt_packages" };
    public const string ManifestFile = "manifest.json";

    public const string TargetPathVar = "target-path";
    public static readonly string[] SourcePathsVar = { "source-paths", "model-paths" };
    public const string MacroPathVar = "macro-paths";

    public const string ResourceTypeModel = "model";
    public const string ResourceTypeSource = "source";
    public const string ResourceTypeSeed = "seed";
    public static readonly string ResourceTypeSnapshot = "snapshot";
    public const string ResourceTypeTest = "test";

    public Uri ProjectRoot { get; }
    public string DbtProfilesDir { get; }
    private string ProjectName { get; set; }
    private string TargetPath { get; set; }
    private List<string> SourcePaths { get; set; }
    private List<string> MacroPaths { get; set; }

    private readonly DbtClient _dbtClient;
    private readonly ILogger _logger;

    public DbtProject(DbtClient dbtClient, ILogger logger)
    {
        _dbtClient = dbtClient;
        _logger = logger;
        RebuildManifest();
        TryRefresh();
    }
    
    private void TryRefresh()
    {
        try
        {
            var projectConfig = ReadAndParseProjectConfig(ProjectRoot);
            ProjectName = projectConfig["name"];
            TargetPath = FindTargetPath(projectConfig);
            SourcePaths = FindSourcePaths(projectConfig);
            MacroPaths = FindMacroPaths(projectConfig);
            // var eventArgs = new ProjectConfigChangedEventArgs
            //     (ProjectRoot, ProjectName, TargetPath, SourcePaths, MacroPaths);
            // OnProjectConfigChanged.Invoke(eventArgs);
        }
        catch (Exception error)
        {
            Console.WriteLine($"An error occurred while trying to refresh the project configuration: {error}");
        }
    }

    public string FindPackageName(Uri uri)
    {
        string documentPath = uri.LocalPath;
        string[] pathSegments = documentPath.Replace(ProjectRoot.LocalPath + Path.DirectorySeparatorChar, "").Split(Path.DirectorySeparatorChar);

        bool insidePackage = pathSegments.Length > 1 && DbtModules.Contains(pathSegments[0]);

        if (insidePackage)
        {
            return pathSegments[1];
        }
        return null;
    }

    public bool Contains(Uri uri)
    {
        return uri.LocalPath.Equals(ProjectRoot.LocalPath, StringComparison.InvariantCultureIgnoreCase) ||
               uri.LocalPath.StartsWith(ProjectRoot.LocalPath + Path.DirectorySeparatorChar, StringComparison.InvariantCultureIgnoreCase);
    }

    private void RebuildManifest()
    {
        try
        {
             _dbtClient.GetProjectKey(ProjectName).Switch(
                 projectKey => _dbtClient
                     .EvalDbtCommand($"to_dict({projectKey}.safe_parse_project())").Switch(
                         _ => { },
                         error =>
                         {
                             _logger.LogError(error.Value, "An error occurred while trying to compile dbt query");
                         }
                     ),
                 _ =>
                 {
                     const string errorMessage = "Could not find project key";
                     _logger.LogError(errorMessage);
                 }
             );   
        }
        catch (Exception exc)
        {
            Console.WriteLine("An error occurred while rebuilding the dbt manifest: " + exc);
        }
    }

    public async Task<OneOf<Success, Error<string>>> RunModel(string processReferenceId, RunModelParams runModelParams, CancellationToken cancellationToken)=>
        await _dbtClient.ExecuteDbtCommand(
            DbtCommandFactory.CreateRunModelCommand(ProjectRoot, DbtProfilesDir, runModelParams, processReferenceId), cancellationToken);
    
    public async Task<OneOf<Success, Error<string>>> BuildModel(string processReferenceId, RunModelParams runModelParams, CancellationToken cancellationToken)=>
        await _dbtClient.ExecuteDbtCommand(
            DbtCommandFactory.CreateBuildModelCommand(ProjectRoot, DbtProfilesDir, runModelParams, processReferenceId), cancellationToken);

    public async Task<OneOf<Success, Error<string>>> RunTest(string processReferenceId, string testName,
        CancellationToken cancellationToken) =>
        await _dbtClient.ExecuteDbtCommand(
            DbtCommandFactory.CreateTestModelCommand(ProjectRoot, DbtProfilesDir, testName, processReferenceId), cancellationToken);

    public async Task<OneOf<Success, Error<string>>> RunModelTest(string processReferenceId, string modelName, CancellationToken cancellationToken) =>
        await _dbtClient.ExecuteDbtCommand(
            DbtCommandFactory.CreateTestModelCommand(ProjectRoot, DbtProfilesDir, modelName, processReferenceId), cancellationToken);

    public async Task<OneOf<Success, Error<string>>> CompileModel(string processReferenceId, RunModelParams runModelParams, CancellationToken cancellationToken) =>
        await _dbtClient.ExecuteDbtCommand(
            DbtCommandFactory.CreateCompileModelCommand(ProjectRoot, DbtProfilesDir, runModelParams, processReferenceId), cancellationToken);

    public OneOf<Success<CompilationResult>, Error<Exception>> CompileQuery(string query) => 
        _dbtClient.GetProjectKey(ProjectName).Match(
            projectKey => _dbtClient
                .EvalDbtCommand($"to_dict({projectKey}.compile_sql({query}))").Match<OneOf<Success<CompilationResult>, Error<Exception>>>(
                    success =>
                    {
                        if (success.Value.IsT1 || success.Value.AsT0.IsNone())
                            return new Error<Exception>(new Exception("Expected value, but received none"));
                
                        return new Success<CompilationResult>(new CompilationResult
                        {
                            CompiledSql = success.Value.AsT0.ToString(CultureInfo.InvariantCulture),
                        });
                    },
                    error =>
                    {
                        _logger.LogError(error.Value, "An error occurred while trying to compile dbt query");
                        return error;
                    }
                ),
            _ =>
            {
                const string errorMessage = "Could not find project key";
                _logger.LogError(errorMessage);
                return new Error<Exception>(new Exception(errorMessage));
            }
        );

    public OneOf<Success<string>, Error<string>> GetCompiledSqlPath(Uri modelPath) => 
        FindModelInTargetfolder(modelPath, "compiled");

    public OneOf<Success<string>, Error<string>> GetRunSqlPath(Uri modelPath) => 
        FindModelInTargetfolder(modelPath, "run");

    public void GenerateModel(string sourceName, string database, string schema, string tableName, string tableIdentifier = null)
    {
        try
        {
            string modelPath = Path.Join(ProjectRoot.LocalPath, SourcePaths[0]);
            string location = Path.Join(modelPath, tableName + ".sql");
            if (!System.IO.File.Exists(location))
            {
                string identifier = tableIdentifier ?? tableName;
                var columnsInRelation =  _dbtClient.GetProjectKey(ProjectName).Match(
                    projectKey => _dbtClient
                        .EvalDbtCommand($"to_dict({projectKey}.get_columns_in_relation(project.create_relation({database}, {schema}, {identifier})))").Match(
                            success =>
                            {
                                if (success.Value.IsT1 || success.Value.AsT0.IsNone())
                                    return success.Value.AsT0.ToString(CultureInfo.InvariantCulture);
                                return "";
                            },
                            error =>
                            {
                                _logger.LogError(error.Value, "An error occurred while trying to compile dbt query");
                                return "";
                            }
                        ),
                    _ => "");
                
                
                Console.WriteLine(columnsInRelation);

                string fileContents = $"with source as (\n  select * from {{ source('{sourceName}', '{tableName}') }}\n),\nrenamed as (\n  select\n    {string.Join(",\n    ", columnsInRelation.Select(column => $"{{ adapter.quote(\"{column}\") }}"))}\n\n  from source\n)\nselect * from renamed\n";
                System.IO.File.WriteAllText(location, fileContents);
                // TODO: Implement opening the created file in the text editor
            }
            else
            {
                Console.WriteLine($"A model called {tableName} already exists in {modelPath}. If you want to generate the model, please rename the other model or delete it if you want to generate the model again.");
            }
        }
        catch (Exception exc)
        {
            Console.WriteLine("An error occurred while trying to generate the model: " + exc.Message);
        }
    }
    
    public static dynamic ReadAndParseProjectConfig(Uri projectRoot)
    {
        string dbtProjectConfigLocation = Path.Join(projectRoot.LocalPath, DbtProjectFile);
        string dbtProjectYamlFile = System.IO.File.ReadAllText(dbtProjectConfigLocation);
        try
        {
            var deserializer = new YamlDotNet.Serialization.DeserializerBuilder()
                .WithNamingConvention(CamelCaseNamingConvention.Instance)
                .Build();

            return deserializer.Deserialize<dynamic>(dbtProjectYamlFile);
        }
        catch (Exception error)
        {
            Console.WriteLine($"Skipping project: could not parse dbt_project_config.yml at '{dbtProjectConfigLocation}': {error}");
            throw;
        }
    }

    private OneOf<Success<string>, Error<string>> FindModelInTargetfolder(Uri modelPath, string type)
    {
        if (string.IsNullOrWhiteSpace(TargetPath))
            return new Error<string>("Target path for DbtProject is not set");
       
        var baseName = Path.GetFileName(modelPath.ToString());
        var found = Directory.GetFiles(TargetPath, $"{type}/**/{baseName}", SearchOption.AllDirectories);
        return found.Length switch
        {
            > 1 => new Error<string>("Found more than one model in target folder"),
            0 => new Error<string>($"Could not find model in target folder: {modelPath}"),
            _ => new Success<string>(found[0])
        };
    }

    private List<string> FindSourcePaths(dynamic projectConfig)
    {
        return SourcePathsVar.Aggregate(new List<string> { "models" }, (prev, current) =>
        {
            if (projectConfig[current] != null)
                return projectConfig[current] as List<string>;
            return prev;
        });
    }

    private List<string> FindMacroPaths(dynamic projectConfig)
    {
        if (projectConfig[MacroPathVar] != null)
            return projectConfig[MacroPathVar] as List<string>;
        return new List<string> { "macros" };
    }

    private string FindTargetPath(dynamic projectConfig)
    {
        if (projectConfig[TargetPathVar] != null)
            return projectConfig[TargetPathVar] as string;
        return "target";
    }
}