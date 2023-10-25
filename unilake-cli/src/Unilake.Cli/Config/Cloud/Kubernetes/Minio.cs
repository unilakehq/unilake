namespace Unilake.Cli.Config;

public class Minio
{
    public bool Enabled { get; set; }
    public string? RootUser { get; set; }
    public string? RootPassword { get; set; }
    public int Replicas { get; set; }
    public List<MinioBucket>? Buckets { get; set; }
}