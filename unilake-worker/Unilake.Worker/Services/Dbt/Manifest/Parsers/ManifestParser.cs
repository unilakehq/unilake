using Newtonsoft.Json;
using Unilake.Worker.Models.Dbt;
using Unilake.Worker.Services.Dbt.Manifest.Event;

namespace Unilake.Worker.Services.Dbt.Manifest.Parsers;

public class ManifestParser
{
    private readonly NodeParser _nodeParser = new();
    private readonly MacroParser _macroParser = new();
    private readonly GraphParser _graphParser = new();
    private readonly SourceParser _sourceParser = new();
    private readonly TestParser _testParser = new();
    private readonly DocParser _docParser = new();
    
    public async Task<ManifestCacheChangedEvent> ParseManifest(
        Uri projectRoot,
        string projectName,
        string targetPath)
    {
        var manifest = ReadAndParseManifest(projectRoot, targetPath);
        if (manifest == null)
        {
            // TODO: better to log an error and handle it gracefully (or we need to do a dbt init in this case, because there is no manifest)
            var eventResult = new ManifestCacheChangedEvent
            {
                Added = new List<ManifestCacheProjectAddedEvent>
                {
                    new()
                    {
                        ProjectName = projectName,
                        ProjectRoot = projectRoot,
                        NodeMetaMap = new NodeMetaMap(),
                        MacroMetaMap = new MacroMetaMap(),
                        SourceMetaMap = new SourceMetaMap(),
                        TestMetaMap = new TestMetaMap(),
                        GraphMetaMap = new GraphMetaMap
                        {
                            Parents = new NodeGraphMap(),
                            Children = new NodeGraphMap(),
                            Tests = new NodeGraphMap(),
                        },
                        DocMetaMap = new DocMetaMap(),
                    },
                },
            };
            return eventResult;
        }

        dynamic nodes = manifest.nodes;
        dynamic sources = manifest.sources;
        dynamic macros = manifest.macros;
        dynamic parentMap = manifest.parent_map;
        dynamic childMap = manifest.child_map;
        dynamic docs = manifest.docs;

        string rootPath = projectRoot.LocalPath;

        var nodeMetaMapPromise = _nodeParser.CreateNodeMetaMap(projectName, nodes, rootPath);
        var macroMetaMapPromise = _macroParser.CreateMacroMetaMap(projectName, macros, rootPath);
        var sourceMetaMapPromise = _sourceParser.CreateSourceMetaMap(sources, rootPath);
        var testMetaMapPromise = _testParser.CreateTestMetaMap(nodes, rootPath);
        var docMetaMapPromise = _docParser.CreateDocMetaMap(docs, projectName, rootPath);

        var results = await Task.WhenAll(nodeMetaMapPromise, macroMetaMapPromise, sourceMetaMapPromise,
            testMetaMapPromise, docMetaMapPromise);

        var nodeMetaMap = results[0];
        var macroMetaMap = results[1];
        var sourceMetaMap = results[2];
        var testMetaMap = results[3];
        var docMetaMap = results[4];

        var graphMetaMap =
            _graphParser.CreateGraphMetaMap(parentMap, childMap, nodeMetaMap, sourceMetaMap, testMetaMap);

        return new ManifestCacheChangedEvent
        {
            Added = new List<ManifestCacheProjectAddedEvent>
            {
                new()
                {
                    ProjectName = projectName,
                    ProjectRoot = projectRoot,
                    NodeMetaMap = nodeMetaMap,
                    MacroMetaMap = macroMetaMap,
                    SourceMetaMap = sourceMetaMap,
                    GraphMetaMap = graphMetaMap,
                    TestMetaMap = testMetaMap,
                    DocMetaMap = docMetaMap,
                },
            },
        };
    }

    private dynamic ReadAndParseManifest(Uri projectRoot, string targetPath)
    {
        string manifestLocation = Path.Combine(projectRoot.LocalPath, targetPath, DbtProject.ManifestFile);
        try
        {
            string manifestFile = System.IO.File.ReadAllText(manifestLocation);
            return JsonConvert.DeserializeObject<dynamic>(manifestFile);
        }
        catch (Exception error)
        {
            // TODO: this
            //_terminal.Log($"Could not read manifest file at {manifestLocation}: {error}");
            return null;
        }
    }

    public static string CreateFullPathForNode(string projectName, string rootPath, string packageName,
        string relativeFilePath)
    {
        if (packageName != projectName)
            return DbtProject.DbtModules
                .Select(modulePathVariant => Path.Combine(rootPath, modulePathVariant, packageName, relativeFilePath))
                .FirstOrDefault(rootPathWithPackage => System.IO.File.Exists(rootPathWithPackage));

        return Path.Combine(rootPath, relativeFilePath);
    }
}
