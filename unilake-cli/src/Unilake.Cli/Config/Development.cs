namespace Unilake.Cli.Config;

public class Development
{
    public bool Enabled { get; set; }
    public KafkaUi KafkaUi { get; set; }
    public Pgweb Pgweb { get; set; }
    public RedisUi RedisUi { get; set; }
    public Gitea Gitea { get; set; }
}