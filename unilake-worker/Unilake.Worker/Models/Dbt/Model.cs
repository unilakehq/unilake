namespace Unilake.Worker.Models.Dbt;

public class Model : Node
{
    public Model(string label, string key, string url) : base(label, key, url) { }
}