namespace Unilake.Worker.Models.Dbt;

public class Snapshot : Node
{
    public Snapshot(string label, string key, string url) : base(label, key, url) { }
}