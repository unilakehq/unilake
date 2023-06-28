namespace Unilake.Worker.Services.Dbt.Command;

public class DbtCommand {
    public string CommandAsString { get; set; }
    public string StatusMessage { get; set; }
    public string Cwd { get; set; }
    public string ProcessReferenceId { get; set; }
}