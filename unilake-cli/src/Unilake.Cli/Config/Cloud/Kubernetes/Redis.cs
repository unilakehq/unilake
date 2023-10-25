namespace Unilake.Cli.Config;

public class Redis
{
    public bool Enabled { get; set; }
    public string Host { get; set; }
    public int? Database { get; set; }
}