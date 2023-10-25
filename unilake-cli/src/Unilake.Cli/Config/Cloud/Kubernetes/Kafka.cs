namespace Unilake.Cli.Config;

public class Kafka
{
    public bool Enabled { get; set; }
    public string Server { get; set; }
    public string SchemaRegistry { get; set; }
}