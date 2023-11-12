using Pulumi;

namespace Unilake.Iac.Kubernetes.Deployment.Input;

public class BoxyHqInputArgs : DeploymentInputArgs
{
    public required Input<string[]> JacksonApiKey { get; set; }

    public string DbEngine { get; set; } = "sql";

    public string DbType { get; set; } = "postgres";

    public required Input<string> DbUsername { get; set; }

    public required Input<string>? DbPassword { get; set; }

    public required Input<string>? DbEndpoint { get; set; }

    public required Input<int> DbPort { get; set; }

    public required Input<string> DbDatabaseName { get; set; }
}