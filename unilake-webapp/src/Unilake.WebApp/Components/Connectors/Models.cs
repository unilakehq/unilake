namespace Unilake.WebApp.Components;

public enum ConnectorType
{
    Streaming,
    Batch
}

public enum ConnectorSource
{
    Airbyte,
    Unilake
}

public enum ConnectorQualityLevel
{
    Gold,
    Silver,
    Bronze
}

public enum ConnectorState
{
    UpToDate,
    Outdated
}
