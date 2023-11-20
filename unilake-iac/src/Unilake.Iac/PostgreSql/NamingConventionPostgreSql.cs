namespace Unilake.Iac.Kubernetes.PostgreSql;

public class NamingConventionPostgreSql : NamingConvention
{
    public string GetDatabaseName() => throw new NotImplementedException();
    
    public string GetSchemaName() => throw new NotImplementedException();
    
    public string GetRoleName() => throw new NotImplementedException();
    public override (bool isSuccess, string errorMessage) IsCompliant(string name, string type)
    {
        throw new NotImplementedException();
    }

    public override string GetAbbreviation<T>()
    {
        throw new NotImplementedException();
    }
}