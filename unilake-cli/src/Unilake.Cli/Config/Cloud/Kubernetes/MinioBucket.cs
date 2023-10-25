namespace Unilake.Cli.Config;

public class MinioBucket
{
    public string Name { get; set; }
    public string Policy { get; set; }
    public bool Purge { get; set; }
    public bool Versioning { get; set; }
    public bool ObjectLocking { get; set; }
}