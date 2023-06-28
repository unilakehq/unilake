namespace Unilake.Worker.Services.Dbt.Manifest.Event;

public class ProjectConfigChangedEvent
{
    public Uri ProjectRoot { get; }
    public string ProjectName { get; }
    public string TargetPath { get; }
    public List<string> SourcePaths { get; }
    public List<string> MacroPaths { get; }

    public ProjectConfigChangedEvent(
        Uri projectRoot,
        string projectName,
        string targetPath,
        List<string> sourcePaths,
        List<string> macroPaths)
    {
        ProjectRoot = projectRoot;
        ProjectName = projectName;
        TargetPath = targetPath;
        SourcePaths = sourcePaths;
        MacroPaths = macroPaths;
    }
}