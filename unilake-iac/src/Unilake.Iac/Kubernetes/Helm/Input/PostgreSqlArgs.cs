using Pulumi;

namespace Unilake.Iac.Kubernetes.Helm.Input;

public class PostgreSqlArgs : HelmInputArgs
{
    public Input<string>? Username { get; set; } = "admin";

    public Input<string>? Password { get; set; } = "";

    public Input<string>? DatabaseName { get; set; }

    public Input<string>? AppName { get; set; } = "general";

    public override string Version { get; set; } = "12.4.1";    


}