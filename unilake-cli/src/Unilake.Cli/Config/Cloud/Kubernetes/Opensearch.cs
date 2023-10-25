namespace Unilake.Cli.Config;

public class Opensearch
{
    public bool Enabled { get; set; }
    public bool SingleNode { get; set; }
    public string Host { get; set; }
    public int? Port { get; set; }
}