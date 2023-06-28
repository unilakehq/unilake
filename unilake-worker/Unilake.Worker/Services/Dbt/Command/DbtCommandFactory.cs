namespace Unilake.Worker.Services.Dbt.Command;

public static class DbtCommandFactory
{
    private static string ProfilesDirParams(string dbtProfilesDir) => "--profiles-dir " + dbtProfilesDir;

    public static DbtCommand CreateVerifyDbtInstalledCommand(string processReferenceId)
    {
        return new DbtCommand
        {
            StatusMessage = "Detecting dbt installation...",
            CommandAsString = "which dbt >/dev/null 2>&1 && echo '1' || echo '0'",
            ProcessReferenceId = processReferenceId,
            Cwd = "/"
        };
    }

    public static DbtCommand CreateVersionCommand(string processReferenceId)
    {
        return new DbtCommand
        {
            StatusMessage = "Detecting dbt version...",
            CommandAsString = "dbt --version",
            ProcessReferenceId = processReferenceId,
            Cwd = "/"
        };
    }

    public static DbtCommand CreateRunModelCommand(Uri projectRoot, string profilesDir, RunModelParams parameters, string processReferenceId)
    {
        var profilesDirParams = ProfilesDirParams(profilesDir);
        return new DbtCommand
        {
            CommandAsString =
                $"dbt run {profilesDirParams} --select {parameters.PlusOperatorLeft}{parameters.ModelName}{parameters.PlusOperatorRight}",
            StatusMessage = "Running dbt models...",
            Cwd = GetCwdFromUri(projectRoot),
            ProcessReferenceId = processReferenceId
        };
    }

    private static string GetCwdFromUri(Uri projectRoot) => projectRoot.LocalPath;

    public static DbtCommand CreateBuildModelCommand(Uri projectRoot, string profilesDir, RunModelParams parameters, string processReferenceId)
    {
        var profilesDirParams = ProfilesDirParams(profilesDir);
        return new DbtCommand
        {
            CommandAsString =
                $"dbt build {profilesDirParams} --select {parameters.PlusOperatorLeft}{parameters.ModelName}{parameters.PlusOperatorRight}",
            StatusMessage = "Building dbt models...",
            Cwd = GetCwdFromUri(projectRoot),
            ProcessReferenceId = processReferenceId
        };
    }

    public static DbtCommand CreateTestModelCommand(Uri projectRoot, string profilesDir, string testName, string processReferenceId)
    {
        var profilesDirParams = ProfilesDirParams(profilesDir);
        return new DbtCommand
        {
            CommandAsString =
                $"dbt test {profilesDirParams} --select {testName}",
            StatusMessage = "Testing dbt model...",
            Cwd = GetCwdFromUri(projectRoot),
            ProcessReferenceId = processReferenceId
        };
    }

    public static DbtCommand CreateCompileModelCommand(Uri projectRoot, string profilesDir, RunModelParams parameters, string processReferenceId)
    {
        var profilesDirParams = ProfilesDirParams(profilesDir);
        return new DbtCommand
        {
            CommandAsString =
                $"dbt compile {profilesDirParams} --model {parameters.PlusOperatorLeft}{parameters.ModelName}{parameters.PlusOperatorRight}",
            StatusMessage = "Compiling dbt models...",
            Cwd = GetCwdFromUri(projectRoot),
            ProcessReferenceId = processReferenceId
        };
    }
}