namespace Unilake.Cli.Config;

public class Kubernetes
{
    public Postgresql Postgresql { get; set; }
    public Opensearch Opensearch { get; set; }
    public Kafka Kafka { get; set; }
    public Storage Storage { get; set; }
    public Karapace Karapace { get; set; }
    public Redis Redis { get; set; }
}