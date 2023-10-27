
namespace Unilake.Cli.Config;

public class Minio : IValidate
{
    public bool Enabled { get; set; }
    public string? RootUser { get; set; }
    public string? RootPassword { get; set; }
    public int Replicas { get; set; } = 1;
    public List<MinioBucket>? Buckets { get; set; }

    public IEnumerable<ValidateResult> Validate(EnvironmentConfig config)
    {
        if(!Enabled)
            yield break;

        if(string.IsNullOrWhiteSpace(RootUser))
            yield return new ValidateResult("Cloud.Minio.RootUser", "RootUser is undefined");
           
        if(string.IsNullOrWhiteSpace(RootPassword))
            yield return new ValidateResult("Cloud.Minio.RootPassword", "RootPassword is undefined");

        if(Replicas < 1)
            yield return new ValidateResult("Cloud.Minio.Replicas", "Replicas cannot be below 1");

        if(Buckets == null || Buckets.Count() < 1)
            yield return new ValidateResult("Cloud.Minio.Buckets", "Buckets cannot be below 1");
        else
            foreach(var err in Buckets.SelectMany(x => x.Validate(config)))
                yield return err;
        
    }
}