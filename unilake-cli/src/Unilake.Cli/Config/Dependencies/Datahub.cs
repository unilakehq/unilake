namespace Unilake.Cli.Config;

public class Datahub
{
    public bool Enabled { get; set; }
    public Postgresql Postgresql { get; set; }
    public Opensearch Opensearch { get; set; }
    public Kafka Kafka { get; set; }
}