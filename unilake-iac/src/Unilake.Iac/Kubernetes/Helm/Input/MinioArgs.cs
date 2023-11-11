using Pulumi;

namespace Unilake.Iac.Kubernetes.Helm.Input;

public class MinioArgs : HelmInputArgs
{
    /// <summary>
    /// List of buckets to be created after minio install
    /// </summary>
    public MinioArgsBucket[] Buckets { get; set; } = Array.Empty<MinioArgsBucket>();

    public Input<string> RootUser { get; set; } = "admin";

    public Input<string> RootPassword { get; set; } = "admin";

    /// <summary>
    /// Number of MinIO containers running (default is 1)
    /// </summary>
    public int Replicas { get; set; } = 1;

    public override string Version { get; set; } = "5.0.8";
}

public class MinioArgsBucket
{
    /// <summary>
    /// Name of the bucket
    /// </summary>
    public required string Name { get; set; }

    /// <summary>
    /// Policy to be set on the bucket
    /// [none|download|upload|public]
    /// </summary>
    public string Policy { get; set; } = "none";

    /// <summary>
    /// Purge if bucket exists already
    /// </summary>
    public bool Purge { get; set; } = false;

    /// <summary>
    /// Enable versioning for bucket
    /// </summary>
    public bool Versioning { get; set; } = false;

    /// <summary>
    /// Set objectlocking for bucket NOTE: versioning is enabled by default if you use locking
    /// </summary>
    public bool ObjectLocking { get; set; } = false;     
}