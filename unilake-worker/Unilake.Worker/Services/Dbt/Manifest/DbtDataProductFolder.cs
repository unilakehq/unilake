using OneOf;
using OneOf.Types;
using Unilake.Worker.Services.Dbt.Command;

namespace Unilake.Worker.Services.Dbt.Manifest;

public class DbtDataProductFolder : IDisposable
{
    private FileSystemWatcher _watcher;
    private readonly DbtClient _dbtClient;
    private readonly string _productFolder;

    public DbtProject DbtProject { get; private set; }

    public DbtDataProductFolder(string productFolder, DbtClient dbtClient)
    {
        _dbtClient = dbtClient;
        _productFolder = productFolder;
        CreateConfigWatcher();
    }

    public OneOf<Success, Error<string>> DiscoverProject()
    {
        var projects = Directory.GetFiles(_productFolder, "dbt_project.yml", SearchOption.AllDirectories);
        if (projects.Length > 1)
            return new Error<string>("Found more than one dbt_project.yml, only one is allowed per data product folder.");
        
        DbtProject = new DbtProject(_dbtClient, null);
        return new Success();
    }
    
    private void CreateConfigWatcher()
    {
        _watcher = new FileSystemWatcher();
        _watcher.Path = _productFolder;
        _watcher.IncludeSubdirectories = true;
        _watcher.Filter = "*.*";

        _watcher.Created += (_, args) => OnFileChange(args);
        _watcher.Deleted += (_, args) => OnFileChange(args);
        _watcher.Renamed += (_, args) => OnFileChange(args);
        _watcher.Changed += (_, args) => OnFileChange(args);
        _watcher.EnableRaisingEvents = true;
    }

    private void OnFileChange(FileSystemEventArgs args)
    {
        // TODO: event when manifest file is changed
        if (args.FullPath.Contains(DbtProject.ManifestFile))
            throw new NotImplementedException();
    }
    
    public void Dispose()
    {
        _watcher?.Dispose();
    }
}