
namespace Unilake.Cli.Config;

public class MinioBucket : IValidate
{
    public string? Name { get; set; }
    public string? Policy { get; set; }
    public bool Purge { get; set; } = false;
    public bool Versioning { get; set; } = false;
    public bool ObjectLocking { get; set; } = false;

    public IEnumerable<ValidateResult> Validate(EnvironmentConfig config)
    {
        if(string.IsNullOrWhiteSpace(Name))
            yield return new ValidateResult("Cloud.Minio.Bucket.Name", "Bucket Name is undefined");
        
        if(string.IsNullOrWhiteSpace(Policy))
            yield return new ValidateResult("Cloud.Minio.Bucket.Policy", "Bucket Policy is undefined");
    }
}