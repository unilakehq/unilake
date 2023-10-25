using Pulumi;

namespace Unilake.Iac.Kubernetes.Deployment.Input;

public class BoxyHqInputArgs : DeploymentInputArgs
{
    public Input<string[]> JacksonApiKey { get; set; }

    public string DbEngine { get; set; } = "sql";

    public string DbType { get; set; } = "postgres";

    public Input<string> DbUsername { get; set; }

    public Input<string> DbPassword { get; set; }

    public Input<string> DbEndpoint { get; set; }

    public Input<int> DbPort { get; set; }

    public Input<string> DbDatabaseName { get; set; }
}