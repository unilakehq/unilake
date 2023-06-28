namespace Unilake.Worker.Models;

public class EnvironmentOptions
{
    private readonly IConfiguration _config;
    public EnvironmentOptions(IConfiguration config) => _config = config;
    
    public string WorkingDirectory => _config.GetValue("environment.workingdirectory", Directory.GetCurrentDirectory());
}
