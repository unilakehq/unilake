namespace Unilake.Worker.Contracts.Requests;

public class AsyncRequestOption
{
    [FromHeader("x-async-request", IsRequired = false, RemoveFromSchema = true)]
    public bool AsyncRequest { get; set; } = true;

    public Mode GetMode() => AsyncRequest ? Mode.WaitForNone : Mode.WaitForAll;
}