namespace Unilake.Cli.Config;

public class EnvironmentConfig
{
    public string Version { get; set; }
    public CloudConfiguration Cloud { get; set; }
    public Components Components { get; set; }
}