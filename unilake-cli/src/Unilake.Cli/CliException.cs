namespace Unilake.Cli;

public sealed class CliException : Exception
{
    public CliException(string message) : base(message)
    {
        
    }
}