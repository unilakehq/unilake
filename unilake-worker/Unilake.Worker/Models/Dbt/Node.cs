namespace Unilake.Worker.Models.Dbt;

public abstract class Node
{
    public string Label { get; set; }
    public string Key { get; set; }
    public string Url { get; set; }
    public IconPath IconPath { get; set; }
    public bool DisplayInModelTree { get; set; }

    protected Node(string label, string key, string url)
    {
        Label = label;
        Key = key;
        Url = url;
        IconPath = new IconPath
        {
            Light = Path.Join(Path.GetDirectoryName(typeof(Node).Assembly.Location), "../media/images/model_light.svg"),
            Dark = Path.Join(Path.GetDirectoryName(typeof(Node).Assembly.Location), "../media/images/model_dark.svg")
        };
        DisplayInModelTree = true;
    }
}