namespace Unilake.Worker.Models.Git;

public class GitOptions
{
    private readonly IConfiguration _config;
    public GitOptions(IConfiguration config) => _config = config;
    public string AccessToken => _config.GetValue("", string.Empty);
    public string Name => _config.GetValue("", string.Empty);
    public string Email => _config.GetValue("", string.Empty);
    public string DefaultBranch => _config.GetValue("", string.Empty);
    public string RepositoryPath => _config.GetValue("", string.Empty);
}