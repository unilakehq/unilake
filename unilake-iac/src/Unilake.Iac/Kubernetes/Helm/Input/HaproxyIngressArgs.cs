namespace Unilake.Iac.Kubernetes.Helm.Input;

public sealed class HaproxyIngressArgs : HelmInputArgs
{
    /// <summary>
    /// can be 'ClusterIP', 'NodePort' or 'LoadBalancer'
    /// </summary>
    public string ServiceType { get; set; } = "LoadBalancer";
    public string IngressClassName { get; set; } = "haproxy";
    public override string Version { get; set; } = "1.30.3";
    /// <summary>
    /// Requires Prometheus Operator to be able to work, default false
    /// </summary>
    public bool EnableServiceMonitor { get; set; } = false;
    /// <summary>
    /// In case nodeports are used as service type (only used for dev/testing environments)
    /// </summary>
    public Dictionary<string, int> NodePorts { get; set; } = new();
    /// <summary>
    /// Number of replicas to deploy for this operator
    /// </summary>
    public int ReplicaCount { get; set; } = 2;
    /// <summary>
    /// If used, set the nodeselector for this helm chart
    /// </summary>
    public Dictionary<string, string> NodeSelector { get; set; } = new ();
    /// <summary>
    /// Additional labels to attach to the service created
    /// </summary>
    public Dictionary<string, string> ServiceLabels { get; set; } = new();
    /// <summary>
    /// In case the service can be disabled, default is enabled
    /// </summary>
    public bool EnableService { get; set; } = true;
    /// <summary>
    /// Set external traffic policy
    /// Default is "Cluster", setting it to "Local" preserves source IP
    /// Ref: https://kubernetes.io/docs/tutorials/services/source-ip/#source-ip-for-services-with-typeloadbalancer
    /// </summary>
    public string ExternalTrafficPolicy { get; set; } = "Cluster";
}